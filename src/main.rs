mod cache;
mod cli;
mod config;
mod detection;
mod ffi;
mod gamemode;
mod games;
mod mangohud;
mod presets;
mod profile;
mod runner;
mod steam;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let cli = cli::Cli::parse();
    let config_manager = config::ConfigManager::new()?;
    let mut config = config_manager.load()?;

    match cli.command {
        cli::Commands::Run(args) => {
            runner::handle_run(args, &config_manager, &mut config)?;
        }
        cli::Commands::Prepare(args) => {
            runner::handle_prepare(args, &config_manager, &mut config)?;
        }
        cli::Commands::Games(args) => {
            games::handle_games(args, &config_manager, &mut config)?;
        }
        cli::Commands::Steam(args) => {
            steam::handle_steam(args, &config_manager, &mut config)?;
        }
        cli::Commands::Detect(args) => {
            detection::handle_detect(args, &config_manager, &mut config)?;
        }
        cli::Commands::Profile(args) => {
            profile::handle_profile(args, &config_manager, &mut config)?;
        }
        cli::Commands::Preset(args) => {
            handle_preset(args, &config_manager)?;
        }
        cli::Commands::Mangohud(args) => {
            handle_mangohud(args)?;
        }
        cli::Commands::Gamemode(args) => {
            handle_gamemode(args)?;
        }
        cli::Commands::Config(args) => {
            config::handle_config(args.command, &config_manager, &mut config)?;
        }
    }

    config_manager.save(&config)?;
    Ok(())
}

fn handle_preset(args: cli::PresetArgs, manager: &config::ConfigManager) -> Result<()> {
    let profile_manager = profile::ProfileManager::new(manager.paths().profiles_dir.clone());

    match args.command {
        cli::PresetCommand::List => {
            println!("Available presets:");
            for preset in presets::PresetType::all() {
                println!("  {} - {}", preset.name(), preset.description());
            }
        }
        cli::PresetCommand::Show { name } => {
            let preset = presets::PresetType::from_name(&name)
                .ok_or_else(|| anyhow::anyhow!("unknown preset: {}", name))?;
            let doc = presets::generate_preset(preset);
            println!("{}", serde_yaml::to_string(&doc)?);
        }
        cli::PresetCommand::Install { force } => {
            let installed = presets::install_presets(&profile_manager, force)?;
            if installed.is_empty() {
                println!("All presets already installed (use --force to overwrite)");
            } else {
                println!("Installed presets: {}", installed.join(", "));
            }
        }
        cli::PresetCommand::Recommend => {
            let preset = presets::recommended_preset();
            let is_deck = presets::is_steam_deck();
            println!("Detected: {}", if is_deck { "Steam Deck" } else { "Desktop" });
            println!("Recommended preset: {}", preset.name());
            println!("Description: {}", preset.description());
        }
    }
    Ok(())
}

fn handle_mangohud(args: cli::MangohudArgs) -> Result<()> {
    match args.command {
        cli::MangohudCommand::Status => {
            let installed = mangohud::is_installed();
            println!("MangoHud installed: {}", if installed { "Yes" } else { "No" });
            if let Some(path) = mangohud::global_config_path() {
                let exists = path.exists();
                println!("Global config: {} ({})",
                    path.display(),
                    if exists { "exists" } else { "not found" });
            }
        }
        cli::MangohudCommand::Generate { preset, output, game } => {
            let mh_preset = match preset.to_lowercase().as_str() {
                "minimal" => mangohud::MangoHudPreset::Minimal,
                "compact" => mangohud::MangoHudPreset::Compact,
                "standard" => mangohud::MangoHudPreset::Standard,
                "full" => mangohud::MangoHudPreset::Full,
                "steam-deck" | "steamdeck" | "deck" => mangohud::MangoHudPreset::SteamDeck,
                "competitive" => mangohud::MangoHudPreset::Competitive,
                "debug" => mangohud::MangoHudPreset::Debug,
                _ => anyhow::bail!("unknown preset: {}", preset),
            };

            let config = mangohud::MangoHudConfig::from_preset(mh_preset);

            let path = if let Some(ref p) = output {
                std::path::PathBuf::from(p)
            } else if let Some(ref g) = game {
                mangohud::game_config_path(g)
                    .ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?
            } else {
                mangohud::global_config_path()
                    .ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?
            };

            config.save(&path)?;
            println!("MangoHud config saved to: {}", path.display());
        }
        cli::MangohudCommand::Env { preset } => {
            let mh_preset = match preset.to_lowercase().as_str() {
                "minimal" => mangohud::MangoHudPreset::Minimal,
                "compact" => mangohud::MangoHudPreset::Compact,
                "standard" | _ => mangohud::MangoHudPreset::Standard,
            };
            let config = mangohud::MangoHudConfig::from_preset(mh_preset);
            for (key, value) in mangohud::env_vars(&config) {
                println!("export {}=\"{}\"", key, value);
            }
        }
    }
    Ok(())
}

fn handle_gamemode(args: cli::GamemodeArgs) -> Result<()> {
    match args.command {
        cli::GamemodeCommand::Status => {
            let installed = gamemode::is_installed();
            println!("GameMode installed: {}", if installed { "Yes" } else { "No" });

            if installed {
                match gamemode::status() {
                    Ok(status) => {
                        println!("Daemon running: {}", if status.running { "Yes" } else { "No" });
                        if status.running {
                            println!("Active clients: {}", status.client_count);
                        }
                    }
                    Err(_) => {
                        println!("Daemon running: No");
                    }
                }
            }

            if let Some(path) = gamemode::config_path() {
                let exists = path.exists();
                println!("Config: {} ({})",
                    path.display(),
                    if exists { "exists" } else { "not found" });
            }
        }
        cli::GamemodeCommand::Generate { config_type, output } => {
            let config = match config_type.to_lowercase().as_str() {
                "default" => gamemode::GameModeConfig::default(),
                "high-performance" | "performance" => gamemode::GameModeConfig::high_performance(),
                "power-save" | "powersave" | "battery" => gamemode::GameModeConfig::power_save(),
                "competitive" | "esports" => gamemode::GameModeConfig::competitive(),
                _ => anyhow::bail!("unknown config type: {}", config_type),
            };

            let path = if let Some(ref p) = output {
                std::path::PathBuf::from(p)
            } else {
                gamemode::config_path()
                    .ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?
            };

            config.save(&path)?;
            println!("GameMode config saved to: {}", path.display());
        }
        cli::GamemodeCommand::Prefix => {
            println!("{}", gamemode::launch_prefix());
        }
    }
    Ok(())
}
