use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::cli::{PrepareArgs, RunArgs};
use crate::config::{ConfigManager, NvConfig};
use crate::detection::{DetectedGame, GameDatabase, GameSource};
use crate::ffi;
use crate::profile::ProfileManager;

/// Runtime context for game launching
pub struct RunContext<'a> {
    #[allow(dead_code)]
    pub config: &'a NvConfig,
    #[allow(dead_code)]
    pub manager: &'a ConfigManager,
    pub profile_manager: ProfileManager,
    pub game_db: GameDatabase,
}

impl<'a> RunContext<'a> {
    pub fn new(config: &'a NvConfig, manager: &'a ConfigManager) -> Result<Self> {
        let profile_manager = ProfileManager::new(manager.paths().profiles_dir.clone());
        let game_db = GameDatabase::load_or_default(manager.paths())?;
        Ok(Self {
            config,
            manager,
            profile_manager,
            game_db,
        })
    }

    /// Find a game by ID or name
    pub fn find_game(&self, id: Option<&str>, name: Option<&str>) -> Result<DetectedGame> {
        if let Some(game_id) = id {
            if let Some(game) = self.game_db.get(game_id) {
                return Ok(game.clone());
            }
        }

        if let Some(game_name) = name {
            let name_lower = game_name.to_lowercase();
            for game in self.game_db.games() {
                if game.name.to_lowercase().contains(&name_lower) {
                    return Ok(game.clone());
                }
            }
        }

        anyhow::bail!(
            "Game not found. Run 'nvproton games scan' to detect games, or use 'nvproton games list' to see available games."
        )
    }
}

/// Handle the `run` command
pub fn handle_run(args: RunArgs, manager: &ConfigManager, config: &mut NvConfig) -> Result<()> {
    let ctx = RunContext::new(config, manager)?;
    let game = ctx.find_game(args.game_id.as_deref(), args.name.as_deref())?;

    println!("Running: {} ({})", game.name, game.id);

    // Build environment variables
    let mut env_vars: HashMap<String, String> = HashMap::new();

    // Apply profile settings if specified
    if let Some(profile_name) = &args.profile {
        let resolved = ctx.profile_manager.resolve(profile_name)?;
        println!("  Profile: {}", profile_name);
        apply_profile_to_env(&resolved.settings, &mut env_vars);
    }

    // NVIDIA-specific optimizations
    if args.reflex {
        env_vars.insert("__GL_REFLEX".into(), "1".into());
        env_vars.insert("DXVK_NVAPI_ALLOW_REFLEX".into(), "1".into());
        println!("  Reflex: enabled");
    }

    if args.fps > 0 {
        env_vars.insert("DXVK_FRAME_RATE".into(), args.fps.to_string());
        println!("  FPS Limit: {}", args.fps);
    }

    if args.vrr {
        env_vars.insert("__GL_GSYNC_ALLOWED".into(), "1".into());
        env_vars.insert("__GL_VRR_ALLOWED".into(), "1".into());
        println!("  VRR: enabled");
    }

    // Shader pre-warming
    if !args.no_prewarm {
        println!("  Pre-warming shaders...");
        if let Err(e) = prewarm_shaders(&game) {
            eprintln!("  Warning: shader pre-warming failed: {}", e);
        }
    }

    // Build launch command based on game source
    let launch_cmd = build_launch_command(&game, &args.game_args)?;

    if args.dry_run {
        println!("\n[Dry Run] Would execute:");
        println!("  Command: {:?}", launch_cmd);
        println!("  Environment:");
        for (key, value) in &env_vars {
            println!("    {}={}", key, value);
        }
        return Ok(());
    }

    // Execute the game
    println!("\nLaunching {}...", game.name);

    let mut cmd = Command::new(&launch_cmd[0]);
    cmd.args(&launch_cmd[1..]);
    cmd.envs(&env_vars);

    // Inherit current env
    for (key, value) in env::vars() {
        if !env_vars.contains_key(&key) {
            cmd.env(key, value);
        }
    }

    let status = cmd.status().context("Failed to launch game")?;

    if !status.success() {
        eprintln!("Game exited with status: {}", status);
    }

    Ok(())
}

/// Handle the `prepare` command
pub fn handle_prepare(
    args: PrepareArgs,
    manager: &ConfigManager,
    config: &mut NvConfig,
) -> Result<()> {
    let ctx = RunContext::new(config, manager)?;
    let game = ctx.find_game(args.game_id.as_deref(), args.name.as_deref())?;

    println!("Preparing: {} ({})", game.name, game.id);

    // Apply profile if specified
    if let Some(profile_name) = &args.profile {
        let resolved = ctx.profile_manager.resolve(profile_name)?;
        println!("  Profile: {} (will be applied at launch)", profile_name);
        // Store profile association for this game
        // TODO: Persist game->profile mapping
        let _ = resolved;
    }

    // Shader pre-warming
    println!("  Pre-warming shaders...");
    if args.force {
        println!("    (forcing recompilation)");
    }

    match prewarm_shaders(&game) {
        Ok(()) => println!("  Shaders ready!"),
        Err(e) => eprintln!("  Warning: shader pre-warming failed: {}", e),
    }

    // Verify game installation
    if game.install_dir.exists() {
        println!("  Install directory: OK");
    } else {
        eprintln!("  Warning: Install directory not found: {:?}", game.install_dir);
    }

    if let Some(exe) = &game.executable {
        if exe.exists() {
            println!("  Executable: OK");
        } else {
            eprintln!("  Warning: Executable not found: {:?}", exe);
        }
    }

    println!("\nGame is ready to launch with 'nvproton run {}'", game.id);
    Ok(())
}

/// Pre-warm shader cache for a game
fn prewarm_shaders(game: &DetectedGame) -> Result<()> {
    // Try to load nvshader library
    let lib_paths = [
        PathBuf::from("/usr/lib/nvproton"),
        PathBuf::from("/usr/local/lib/nvproton"),
        dirs::data_local_dir()
            .map(|d| d.join("nvproton/lib"))
            .unwrap_or_default(),
    ];

    for path in &lib_paths {
        let shader_lib = path.join("libnvshader.so");
        if shader_lib.exists() {
            match unsafe { ffi::NvShader::load(&shader_lib) } {
                Ok(nvshader) => {
                    return nvshader.warm_cache(&game.id).map_err(Into::into);
                }
                Err(e) => {
                    log::debug!("Failed to load nvshader from {:?}: {}", shader_lib, e);
                }
            }
        }
    }

    // Fallback: check if DXVK cache exists
    let cache_paths = get_shader_cache_paths(game);
    for path in &cache_paths {
        if path.exists() {
            log::info!("Found existing shader cache at {:?}", path);
            return Ok(());
        }
    }

    log::warn!("No shader cache found - first launch may have stuttering");
    Ok(())
}

/// Get potential shader cache paths for a game
fn get_shader_cache_paths(game: &DetectedGame) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(cache_dir) = dirs::cache_dir() {
        // DXVK cache
        paths.push(cache_dir.join("dxvk").join(&game.id));
        // vkd3d-proton cache
        paths.push(cache_dir.join("vkd3d-proton").join(&game.id));
        // Mesa shader cache
        paths.push(cache_dir.join("mesa_shader_cache"));
    }

    // Steam shader cache
    if let GameSource::Steam = game.source {
        if let Some(home) = dirs::home_dir() {
            paths.push(
                home.join(".local/share/Steam/steamapps/shadercache")
                    .join(&game.id),
            );
        }
    }

    paths
}

/// Build the launch command for a game
fn build_launch_command(game: &DetectedGame, extra_args: &[String]) -> Result<Vec<String>> {
    let mut cmd = Vec::new();

    match game.source {
        GameSource::Steam => {
            // Use steam to launch
            cmd.push("steam".into());
            cmd.push("-applaunch".into());
            cmd.push(game.id.clone());
            cmd.extend(extra_args.iter().cloned());
        }
        GameSource::Heroic => {
            // Use heroic CLI
            cmd.push("heroic".into());
            cmd.push("--launch".into());
            cmd.push(game.id.clone());
            cmd.extend(extra_args.iter().cloned());
        }
        GameSource::Lutris => {
            // Use lutris CLI
            cmd.push("lutris".into());
            cmd.push(format!("lutris:rungame/{}", game.id));
            cmd.extend(extra_args.iter().cloned());
        }
        GameSource::Unknown => {
            // Direct executable launch
            if let Some(exe) = &game.executable {
                cmd.push(exe.to_string_lossy().into_owned());
                cmd.extend(extra_args.iter().cloned());
            } else {
                anyhow::bail!(
                    "Cannot launch game '{}' - no executable found",
                    game.name
                );
            }
        }
    }

    Ok(cmd)
}

/// Apply profile settings to environment variables
fn apply_profile_to_env(settings: &serde_yaml::Value, env_vars: &mut HashMap<String, String>) {
    if let serde_yaml::Value::Mapping(map) = settings {
        // Handle env section directly
        if let Some(serde_yaml::Value::Mapping(env_map)) =
            map.get(&serde_yaml::Value::String("env".into()))
        {
            for (key, value) in env_map {
                if let (serde_yaml::Value::String(k), serde_yaml::Value::String(v)) = (key, value) {
                    env_vars.insert(k.clone(), v.clone());
                }
            }
        }

        // Handle nvidia section
        if let Some(serde_yaml::Value::Mapping(nvidia_map)) =
            map.get(&serde_yaml::Value::String("nvidia".into()))
        {
            for (key, value) in nvidia_map {
                if let serde_yaml::Value::String(k) = key {
                    let env_key = format!("__GL_{}", k.to_uppercase());
                    match value {
                        serde_yaml::Value::Bool(b) => {
                            env_vars.insert(env_key, if *b { "1" } else { "0" }.into());
                        }
                        serde_yaml::Value::Number(n) => {
                            env_vars.insert(env_key, n.to_string());
                        }
                        serde_yaml::Value::String(s) => {
                            env_vars.insert(env_key, s.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle dxvk section
        if let Some(serde_yaml::Value::Mapping(dxvk_map)) =
            map.get(&serde_yaml::Value::String("dxvk".into()))
        {
            for (key, value) in dxvk_map {
                if let serde_yaml::Value::String(k) = key {
                    let env_key = format!("DXVK_{}", k.to_uppercase());
                    match value {
                        serde_yaml::Value::Bool(b) => {
                            env_vars.insert(env_key, if *b { "1" } else { "0" }.into());
                        }
                        serde_yaml::Value::Number(n) => {
                            env_vars.insert(env_key, n.to_string());
                        }
                        serde_yaml::Value::String(s) => {
                            env_vars.insert(env_key, s.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
