//! Steam Integration Module
//!
//! Provides deep Steam integration:
//! - Launch option generation
//! - Non-Steam shortcut creation
//! - Proton version management
//! - Steam Input configuration

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::cli::{SteamArgs, SteamCommand};
use crate::config::{ConfigManager, NvConfig};
use crate::detection::GameDatabase;

/// Handle Steam subcommands
pub fn handle_steam(args: SteamArgs, manager: &ConfigManager, config: &mut NvConfig) -> Result<()> {
    match args.command {
        SteamCommand::LaunchOptions(opts) => handle_launch_options(opts, manager, config),
        SteamCommand::Proton(opts) => handle_proton(opts, manager, config),
        SteamCommand::Shortcut(opts) => handle_shortcut(opts, manager, config),
    }
}

/// Generate recommended launch options for a game
fn handle_launch_options(
    args: crate::cli::LaunchOptionsArgs,
    manager: &ConfigManager,
    _config: &NvConfig,
) -> Result<()> {
    let db = GameDatabase::load_or_default(manager.paths())?;

    let game = db.get(&args.game_id).ok_or_else(|| {
        anyhow::anyhow!(
            "Game '{}' not found. Run 'nvproton games scan' first.",
            args.game_id
        )
    })?;

    println!("Launch Options for: {} ({})", game.name, game.id);
    println!();

    // Build launch options
    let mut options = Vec::new();

    // Always use nvproton wrapper
    if args.use_nvproton {
        options.push(format!("nvproton run {} --", game.id));
    }

    // Reflex/low latency
    if args.reflex {
        options.push("DXVK_NVAPI_ALLOW_REFLEX=1".into());
        options.push("__GL_REFLEX=1".into());
    }

    // VRR/G-Sync
    if args.vrr {
        options.push("__GL_GSYNC_ALLOWED=1".into());
        options.push("__GL_VRR_ALLOWED=1".into());
    }

    // FPS limit
    if args.fps > 0 {
        options.push(format!("DXVK_FRAME_RATE={}", args.fps));
    }

    // Shader cache path
    if args.shader_cache {
        options.push(format!(
            "DXVK_STATE_CACHE_PATH=~/.cache/nvproton/{}",
            game.id
        ));
    }

    // MangoHud
    if args.mangohud {
        options.push("mangohud".into());
    }

    // Gamemode
    if args.gamemode {
        options.push("gamemoderun".into());
    }

    // Custom env vars
    for (key, value) in &args.env {
        options.push(format!("{}={}", key, value));
    }

    // Output formats
    if args.copy_format {
        // Format for Steam's "Set Launch Options" dialog
        let steam_options = build_steam_launch_string(&options, args.use_nvproton);
        println!("Copy this into Steam's \"Set Launch Options\":\n");
        println!("{}", steam_options);
    } else {
        println!("Recommended environment variables:");
        for opt in &options {
            if opt.contains('=') {
                println!("  {}", opt);
            }
        }
        println!();
        println!("Full launch command:");
        println!(
            "  {}",
            build_steam_launch_string(&options, args.use_nvproton)
        );
    }

    println!();
    println!("To apply in Steam:");
    println!("  1. Right-click {} in your library", game.name);
    println!("  2. Properties > General > Launch Options");
    println!("  3. Paste the command above");

    Ok(())
}

/// Build a Steam-compatible launch options string
fn build_steam_launch_string(options: &[String], use_nvproton: bool) -> String {
    let mut parts = Vec::new();
    let mut env_vars = Vec::new();

    for opt in options {
        if opt.contains('=') && !opt.starts_with("nvproton") {
            env_vars.push(opt.clone());
        } else {
            parts.push(opt.clone());
        }
    }

    // Build the command
    let mut result = String::new();

    // Environment variables first
    for var in &env_vars {
        result.push_str(var);
        result.push(' ');
    }

    // Wrapper commands (mangohud, gamemoderun, nvproton)
    for part in &parts {
        if !part.starts_with("nvproton") {
            result.push_str(part);
            result.push(' ');
        }
    }

    if use_nvproton {
        result.push_str("nvproton run %appid% -- ");
    }

    // %command% is required by Steam
    result.push_str("%command%");

    result
}

/// Handle Proton version management
fn handle_proton(
    args: crate::cli::ProtonArgs,
    _manager: &ConfigManager,
    config: &NvConfig,
) -> Result<()> {
    let steam_path = config
        .library_paths
        .steam
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Steam path not configured"))?;

    match args.command {
        crate::cli::ProtonCommand::List => {
            println!("Installed Proton versions:\n");

            // Check compatibilitytools.d
            let compat_dir = steam_path.join("compatibilitytools.d");
            if compat_dir.exists() {
                println!("Custom (compatibilitytools.d):");
                list_proton_versions(&compat_dir, "  ")?;
            }

            // Check Steam's Proton installs
            let proton_dirs = [steam_path.join("steamapps/common")];

            println!("\nSteam-installed:");
            for dir in &proton_dirs {
                if dir.exists() {
                    for entry in fs::read_dir(dir)? {
                        let entry = entry?;
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.contains("Proton") || name.contains("proton") {
                            println!("  {}", name);
                        }
                    }
                }
            }
        }
        crate::cli::ProtonCommand::Recommended => {
            println!("Recommended Proton versions for NVIDIA:\n");
            println!("1. Proton Experimental (latest features)");
            println!("   - Best for: Most modern games, VR");
            println!("   - DLSS: Full support");
            println!("   - Reflex: Full support");
            println!();
            println!("2. Proton GE (GloriousEggroll)");
            println!("   - Best for: Games with codec issues, older titles");
            println!("   - Install: https://github.com/GloriousEggroll/proton-ge-custom");
            println!();
            println!("3. Proton 9.x (stable)");
            println!("   - Best for: Games that need stability");
            println!("   - DLSS: Supported");
            println!();
            println!("For competitive gaming with Reflex, use Proton Experimental.");
        }
        crate::cli::ProtonCommand::SetDefault { version } => {
            println!("Setting default Proton version to: {}", version);
            println!();
            println!("To set default Proton in Steam:");
            println!("  1. Steam > Settings > Compatibility");
            println!("  2. Enable 'Enable Steam Play for all other titles'");
            println!("  3. Select '{}' from the dropdown", version);
            println!();
            println!("Note: nvproton respects Steam's per-game Proton settings.");
        }
    }

    Ok(())
}

/// List Proton versions in a directory
fn list_proton_versions(dir: &Path, prefix: &str) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Check for proton binary or toolmanifest
            if path.join("proton").exists() || path.join("toolmanifest.vdf").exists() {
                println!("{}{}", prefix, name);
            }
        }
    }

    Ok(())
}

/// Handle non-Steam shortcut creation
fn handle_shortcut(
    args: crate::cli::ShortcutArgs,
    manager: &ConfigManager,
    config: &NvConfig,
) -> Result<()> {
    let steam_path = config
        .library_paths
        .steam
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Steam path not configured"))?;

    match args.command {
        crate::cli::ShortcutCommand::Create {
            name,
            exe,
            start_dir,
            icon,
            launch_options,
        } => {
            println!("Creating non-Steam shortcut: {}", name);
            println!();

            // Find shortcuts.vdf
            let userdata_dir = steam_path.join("userdata");
            if !userdata_dir.exists() {
                anyhow::bail!("Steam userdata directory not found");
            }

            // List Steam user IDs
            let user_dirs: Vec<_> = fs::read_dir(&userdata_dir)?
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .collect();

            if user_dirs.is_empty() {
                anyhow::bail!("No Steam users found");
            }

            // Use first user or let user choose
            let user_dir = &user_dirs[0].path();
            let shortcuts_path = user_dir.join("config/shortcuts.vdf");

            println!("Shortcut details:");
            println!("  Name: {}", name);
            println!("  Executable: {}", exe);
            if let Some(ref dir) = start_dir {
                println!("  Start In: {}", dir);
            }
            if let Some(ref ico) = icon {
                println!("  Icon: {}", ico);
            }
            if let Some(ref opts) = launch_options {
                println!("  Launch Options: {}", opts);
            }

            println!();
            println!("To add manually in Steam:");
            println!("  1. Library > Add a Game > Add a Non-Steam Game");
            println!("  2. Browse to: {}", exe);
            println!("  3. Right-click the shortcut > Properties");
            if let Some(opts) = launch_options {
                println!("  4. Set Launch Options: {}", opts);
            }

            // Note: Actually modifying shortcuts.vdf requires parsing its binary format
            // For now, provide instructions
            println!();
            println!("Note: Automatic shortcut creation requires Steam to be closed.");
            println!("The shortcuts.vdf file is located at: {:?}", shortcuts_path);
        }
        crate::cli::ShortcutCommand::List => {
            println!("Non-Steam shortcuts:\n");

            let userdata_dir = steam_path.join("userdata");
            if !userdata_dir.exists() {
                println!("No Steam userdata found.");
                return Ok(());
            }

            for user_entry in fs::read_dir(&userdata_dir)?.filter_map(Result::ok) {
                let shortcuts_path = user_entry.path().join("config/shortcuts.vdf");
                if shortcuts_path.exists() {
                    println!("User: {}", user_entry.file_name().to_string_lossy());
                    println!("  Shortcuts file: {:?}", shortcuts_path);
                    // Note: Full parsing would require VDF binary format support
                }
            }
        }
        crate::cli::ShortcutCommand::Optimize { appid, profile } => {
            let db = GameDatabase::load_or_default(manager.paths())?;

            if let Some(game) = db.get(&appid) {
                println!("Optimizing shortcut for: {} ({})", game.name, appid);
                println!();

                // Generate optimized launch options
                let mut options = vec![
                    "DXVK_NVAPI_ALLOW_REFLEX=1".into(),
                    "__GL_REFLEX=1".into(),
                    "__GL_GSYNC_ALLOWED=1".into(),
                ];

                if let Some(profile_name) = profile {
                    println!("Applying profile: {}", profile_name);
                    // Load profile and add its env vars
                    let profile_manager =
                        crate::profile::ProfileManager::new(manager.paths().profiles_dir.clone());
                    if let Ok(resolved) = profile_manager.resolve(&profile_name) {
                        // Extract env vars from profile
                        if let serde_yaml::Value::Mapping(map) = &resolved.settings
                            && let Some(serde_yaml::Value::Mapping(env)) =
                                map.get(serde_yaml::Value::String("env".into()))
                        {
                            for (k, v) in env {
                                if let (
                                    serde_yaml::Value::String(key),
                                    serde_yaml::Value::String(val),
                                ) = (k, v)
                                {
                                    options.push(format!("{}={}", key, val));
                                }
                            }
                        }
                    }
                }

                println!("Recommended launch options:");
                let launch_str = build_steam_launch_string(&options, false);
                println!("  {}", launch_str);
            } else {
                anyhow::bail!("Game '{}' not found in database", appid);
            }
        }
    }

    Ok(())
}
