use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::cli::{DescriptorHeapMode, PrepareArgs, RunArgs};
use crate::config::{ConfigManager, NvConfig};
use crate::detection::proton_nv::{ProtonNvDetector, ProtonNvEnv, ProtonNvInstallation};
use crate::detection::{DetectedGame, GameDatabase, GameSource, VulkanCapabilities};
use crate::ffi;
use crate::profile::{ProfileManager, ProfilePersistence};

/// Runtime context for game launching
pub struct RunContext<'a> {
    #[allow(dead_code)]
    pub config: &'a NvConfig,
    #[allow(dead_code)]
    pub manager: &'a ConfigManager,
    pub profile_manager: ProfileManager,
    pub profile_persistence: ProfilePersistence,
    pub game_db: GameDatabase,
    pub proton_nv: Option<ProtonNvInstallation>,
    pub vulkan_caps: Option<VulkanCapabilities>,
}

impl<'a> RunContext<'a> {
    pub fn new(config: &'a NvConfig, manager: &'a ConfigManager) -> Result<Self> {
        let profile_manager = ProfileManager::new(manager.paths().profiles_dir.clone());
        let db_path = manager.paths().user_config_dir.join("profiles.db");
        let profile_persistence = ProfilePersistence::open(&db_path)
            .context("failed to open profile persistence database")?;
        let game_db = GameDatabase::load_or_default(manager.paths())?;

        // Detect Proton-NV installation
        let proton_nv = {
            let mut detector = ProtonNvDetector::new();
            match detector.scan() {
                Ok(_) => detector.get_best().cloned(),
                Err(e) => {
                    log::debug!("Proton-NV detection failed: {}", e);
                    None
                }
            }
        };

        if let Some(ref pnv) = proton_nv {
            log::info!("Proton-NV detected: {} at {:?}", pnv.version, pnv.path);
        }

        // Detect Vulkan capabilities (for descriptor_heap support)
        let vulkan_caps = match VulkanCapabilities::detect() {
            Ok(caps) => {
                log::info!(
                    "Vulkan: {} (driver {})",
                    caps.gpu_name,
                    caps.driver_version
                );
                if caps.descriptor_heap {
                    log::info!("VK_EXT_descriptor_heap: supported");
                }
                Some(caps)
            }
            Err(e) => {
                log::debug!("Vulkan capability detection failed: {}", e);
                None
            }
        };

        Ok(Self {
            config,
            manager,
            profile_manager,
            profile_persistence,
            game_db,
            proton_nv,
            vulkan_caps,
        })
    }

    /// Find a game by ID or name
    pub fn find_game(&self, id: Option<&str>, name: Option<&str>) -> Result<DetectedGame> {
        if let Some(game_id) = id
            && let Some(game) = self.game_db.get(game_id)
        {
            return Ok(game.clone());
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

    // Apply Proton-NV optimizations if available
    if let Some(ref proton_nv) = ctx.proton_nv {
        println!("  Proton-NV: {} detected", proton_nv.version);
        let pnv_env = ProtonNvEnv::from_installation(proton_nv);
        for (key, value) in pnv_env.vars() {
            env_vars.insert(key.clone(), value.clone());
        }
    }

    // Determine which profile to use: command-line arg takes precedence over persisted binding
    let profile_name = if let Some(name) = &args.profile {
        Some(name.clone())
    } else {
        // Check for persisted profile binding
        ctx.profile_persistence.get_binding(&game.id).ok().flatten()
    };

    // Apply profile settings
    if let Some(profile_name) = &profile_name {
        let resolved = ctx.profile_manager.resolve(profile_name)?;
        println!("  Profile: {}", profile_name);
        apply_profile_to_env(&resolved.settings, &mut env_vars);
    }

    // NVIDIA-specific optimizations via FFI
    // Configure Reflex via nvlatency library
    if args.reflex {
        // Check for Reflex 2.0 support (VK_NV_low_latency2 on 595+)
        let has_reflex2 = ctx.vulkan_caps.as_ref().is_some_and(|c| c.supports_reflex2());

        // Set environment variables as fallback for DXVK/Wine
        env_vars.insert("__GL_REFLEX".into(), "1".into());
        env_vars.insert("DXVK_NVAPI_ALLOW_REFLEX".into(), "1".into());

        // Enable Reflex 2.0 features if available
        if has_reflex2 {
            env_vars.insert("__GL_REFLEX_MODE".into(), "2".into()); // Reflex 2.0 mode
        }

        // Also configure via FFI for native applications
        if let Err(e) = configure_reflex(true) {
            log::warn!("Reflex FFI configuration failed: {}", e);
            if has_reflex2 {
                println!("  Reflex 2.0: enabled (env vars only)");
            } else {
                println!("  Reflex: enabled (env vars only)");
            }
        } else if has_reflex2 {
            println!("  Reflex 2.0: enabled via nvlatency");
        }
    }

    // Configure VRR and frame limiting via nvsync library
    if args.fps > 0 {
        env_vars.insert("DXVK_FRAME_RATE".into(), args.fps.to_string());
    }

    if args.vrr {
        env_vars.insert("__GL_GSYNC_ALLOWED".into(), "1".into());
        env_vars.insert("__GL_VRR_ALLOWED".into(), "1".into());
    }

    // Configure via FFI for system-level VRR and frame limiting
    if args.vrr || args.fps > 0 {
        if let Err(e) = configure_vrr(args.vrr, args.fps) {
            log::warn!("VRR/FPS FFI configuration failed: {}", e);
            if args.vrr {
                println!("  VRR: enabled (env vars only)");
            }
            if args.fps > 0 {
                println!("  FPS Limit: {} (env vars only)", args.fps);
            }
        }
    }

    // Configure VK_EXT_descriptor_heap for DX12 games
    let has_descriptor_heap = ctx
        .vulkan_caps
        .as_ref()
        .is_some_and(|c| c.supports_descriptor_heap());
    let has_heap_fix = ctx
        .vulkan_caps
        .as_ref()
        .is_some_and(|c| c.supports_dx12_heap_fix());
    let is_595 = ctx.vulkan_caps.as_ref().is_some_and(|c| c.is_595_series());

    let use_descriptor_heap = match args.descriptor_heap {
        DescriptorHeapMode::On => true,
        DescriptorHeapMode::Off => false,
        DescriptorHeapMode::Auto => {
            // Auto-enable on 595+ if config allows, or if extension is available
            (config.vkd3d.auto_enable_595 && is_595) || has_descriptor_heap
        }
    };

    if use_descriptor_heap {
        // Build VKD3D_CONFIG with all relevant flags
        let vkd3d_config =
            config
                .vkd3d
                .build_config_string(has_descriptor_heap, has_heap_fix);
        if !vkd3d_config.is_empty() {
            env_vars.insert("VKD3D_CONFIG".into(), vkd3d_config);
        }
        env_vars.insert("VKD3D_FEATURE_LEVEL".into(), config.vkd3d.feature_level.clone());

        if has_heap_fix {
            println!("  Descriptor Heap: enabled (DX12 optimization + 595 heap fix)");
        } else {
            println!("  Descriptor Heap: enabled (DX12 optimization)");
        }
    }

    // Warn about beta driver if configured (but 595 is recommended so note that)
    if let Some(ref caps) = ctx.vulkan_caps {
        if caps.is_beta_driver() && config.vkd3d.warn_beta_driver {
            if caps.is_595_series() {
                eprintln!(
                    "  Note: 595 beta driver {} - recommended for DX12 games (heap fixes included)",
                    caps.driver_version
                );
            } else {
                eprintln!(
                    "  Warning: Beta driver {} detected. Consider updating to 595.x for heap fixes.",
                    caps.driver_version
                );
            }
        }
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

    // Report Proton-NV status
    if let Some(ref proton_nv) = ctx.proton_nv {
        println!("  Proton-NV: {} (will be used at launch)", proton_nv.version);
        if let Some(ref info) = proton_nv.version_info {
            if let Some(ref driver) = info.nvidia_driver_min {
                println!("    Requires: NVIDIA driver {}", driver);
            }
            if let Some(ref gpu) = info.target_gpu {
                println!("    Target: {}", gpu);
            }
        }
    } else {
        println!("  Proton-NV: not detected (using system Proton)");
    }

    // Apply profile if specified
    if let Some(profile_name) = &args.profile {
        // Verify profile exists by resolving it
        let _resolved = ctx.profile_manager.resolve(profile_name)?;
        // Persist game->profile binding
        ctx.profile_persistence.bind(&game.id, profile_name)
            .with_context(|| format!("failed to bind profile '{}' to game '{}'", profile_name, game.id))?;
        println!("  Profile: {} (bound to game, will be applied at launch)", profile_name);
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
        eprintln!(
            "  Warning: Install directory not found: {:?}",
            game.install_dir
        );
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

/// Configure Reflex low-latency mode using nvlatency library
fn configure_reflex(enabled: bool) -> Result<()> {
    if !enabled {
        return Ok(());
    }

    let lib_paths = get_lib_paths();

    for path in &lib_paths {
        let latency_lib = path.join("libnvlatency.so");
        if latency_lib.exists() {
            match unsafe { ffi::NvLatency::load(&latency_lib) } {
                Ok(nvlatency) => {
                    // Check if NVIDIA GPU is present
                    if !nvlatency.is_nvidia_gpu() {
                        log::warn!("Reflex requires NVIDIA GPU");
                        return Ok(());
                    }

                    // Check if Reflex is supported
                    if !nvlatency.is_supported() {
                        log::info!("Reflex not supported on this configuration");
                        return Ok(());
                    }

                    // Enable Reflex in On mode (not Boost, as that's more aggressive)
                    if let Err(e) = nvlatency.set_reflex_mode(ffi::ReflexMode::On) {
                        log::warn!("Failed to enable Reflex: {}", e);
                    } else {
                        println!("  Reflex: enabled via nvlatency");
                    }
                    return Ok(());
                }
                Err(e) => {
                    log::debug!("Failed to load nvlatency from {:?}: {}", latency_lib, e);
                }
            }
        }
    }

    log::debug!("nvlatency library not found - Reflex FFI unavailable");
    Ok(())
}

/// Configure VRR (G-Sync/FreeSync) and frame limiter using nvsync library
fn configure_vrr(enabled: bool, fps_limit: u32) -> Result<()> {
    // Skip if nothing to configure
    if !enabled && fps_limit == 0 {
        return Ok(());
    }

    let lib_paths = get_lib_paths();

    for path in &lib_paths {
        let sync_lib = path.join("libnvsync.so");
        if sync_lib.exists() {
            match unsafe { ffi::NvSync::load(&sync_lib) } {
                Ok(nvsync) => {
                    // Scan for displays
                    if let Err(e) = nvsync.scan() {
                        log::warn!("Failed to scan displays: {}", e);
                        return Ok(());
                    }

                    // Get system status
                    if let Ok(status) = nvsync.get_status() {
                        if !status.nvidia_detected {
                            log::warn!("VRR requires NVIDIA GPU");
                            return Ok(());
                        }

                        if status.vrr_capable_count == 0 {
                            log::info!("No VRR-capable displays detected");
                            return Ok(());
                        }
                    }

                    // Enable VRR if requested
                    if enabled {
                        if let Err(e) = nvsync.enable_vrr(None) {
                            log::warn!("Failed to enable VRR: {}", e);
                        } else {
                            println!("  VRR: enabled via nvsync");
                        }
                    }

                    // Set frame limit if requested
                    if fps_limit > 0 {
                        if let Err(e) = nvsync.set_frame_limit(fps_limit) {
                            log::warn!("Failed to set frame limit: {}", e);
                        } else {
                            println!("  Frame limit: {} FPS via nvsync", fps_limit);
                        }
                    }

                    return Ok(());
                }
                Err(e) => {
                    log::debug!("Failed to load nvsync from {:?}: {}", sync_lib, e);
                }
            }
        }
    }

    log::debug!("nvsync library not found - VRR FFI unavailable");
    Ok(())
}

/// Get standard library search paths
fn get_lib_paths() -> Vec<PathBuf> {
    let mut lib_paths = vec![
        PathBuf::from("/usr/lib/nvproton"),
        PathBuf::from("/usr/local/lib/nvproton"),
        PathBuf::from("/usr/lib"),
        PathBuf::from("/usr/local/lib"),
        dirs::data_local_dir()
            .map(|d| d.join("nvproton/lib"))
            .unwrap_or_default(),
    ];

    // Prepend custom path from environment if set
    if let Ok(custom_path) = env::var("NVPROTON_LIB_PATH") {
        lib_paths.insert(0, PathBuf::from(custom_path));
    }

    lib_paths
}

/// Pre-warm shader cache for a game using nvshader library
fn prewarm_shaders(game: &DetectedGame) -> Result<()> {
    let lib_paths = get_lib_paths();

    for path in &lib_paths {
        let shader_lib = path.join("libnvshader.so");
        if shader_lib.exists() {
            match unsafe { ffi::NvShader::load(&shader_lib) } {
                Ok(nvshader) => {
                    // Check if pre-warming is available (fossilize_replay found)
                    if !nvshader.prewarm_available() {
                        log::info!("fossilize_replay not available - skipping shader pre-warm");
                        return Ok(());
                    }

                    // Scan for caches first
                    if let Err(e) = nvshader.scan() {
                        log::warn!("Failed to scan shader caches: {}", e);
                        return Ok(());
                    }

                    // Pre-warm shaders for this game
                    match nvshader.prewarm_game(&game.id) {
                        Ok(result) => {
                            if result.total > 0 {
                                println!(
                                    "  Shaders: {}/{} compiled ({} failed, {} skipped)",
                                    result.completed, result.total, result.failed, result.skipped
                                );
                            } else {
                                println!("  Shaders: No Fossilize caches found for this game");
                            }
                            return Ok(());
                        }
                        Err(ffi::FfiError::Operation { code: -5 }) => {
                            // Game not found in caches - that's OK
                            log::debug!("No shader cache found for game {}", game.id);
                        }
                        Err(e) => {
                            log::warn!("Failed to pre-warm shaders: {}", e);
                        }
                    }
                    return Ok(());
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

    log::debug!("nvshader library not found - shader pre-warming unavailable");
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
    if let GameSource::Steam = game.source
        && let Some(home) = dirs::home_dir()
    {
        paths.push(
            home.join(".local/share/Steam/steamapps/shadercache")
                .join(&game.id),
        );
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
                anyhow::bail!("Cannot launch game '{}' - no executable found", game.name);
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
            map.get(serde_yaml::Value::String("env".into()))
        {
            for (key, value) in env_map {
                if let (serde_yaml::Value::String(k), serde_yaml::Value::String(v)) = (key, value) {
                    env_vars.insert(k.clone(), v.clone());
                }
            }
        }

        // Handle nvidia section
        if let Some(serde_yaml::Value::Mapping(nvidia_map)) =
            map.get(serde_yaml::Value::String("nvidia".into()))
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
            map.get(serde_yaml::Value::String("dxvk".into()))
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

        // Handle vkd3d section
        if let Some(serde_yaml::Value::Mapping(vkd3d_map)) =
            map.get(serde_yaml::Value::String("vkd3d".into()))
        {
            // Handle descriptor_heap setting
            if let Some(serde_yaml::Value::String(mode)) =
                vkd3d_map.get(&serde_yaml::Value::String("descriptor_heap".into()))
            {
                match mode.as_str() {
                    "on" | "enabled" | "true" => {
                        env_vars.insert("VKD3D_CONFIG".into(), "descriptor_heap".into());
                        env_vars.insert("VKD3D_FEATURE_LEVEL".into(), "12_2".into());
                    }
                    "off" | "disabled" | "false" => {
                        // Remove if previously set
                        env_vars.remove("VKD3D_CONFIG");
                    }
                    // "auto" - don't override, let runtime detection handle it
                    _ => {}
                }
            }

            // Handle config setting (VKD3D_CONFIG value)
            if let Some(serde_yaml::Value::String(config_val)) =
                vkd3d_map.get(&serde_yaml::Value::String("config".into()))
            {
                env_vars.insert("VKD3D_CONFIG".into(), config_val.clone());
            }

            // Handle feature_level setting
            if let Some(serde_yaml::Value::String(level)) =
                vkd3d_map.get(&serde_yaml::Value::String("feature_level".into()))
            {
                env_vars.insert("VKD3D_FEATURE_LEVEL".into(), level.clone());
            }
        }
    }
}
