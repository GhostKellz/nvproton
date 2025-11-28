mod cli;
mod config;
mod detection;
mod ffi;
mod profile;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let cli = cli::Cli::parse();
    let config_manager = config::ConfigManager::new()?;
    let mut config = config_manager.load()?;

    match cli.command {
        cli::Commands::Detect(args) => {
            detection::handle_detect(args, &config_manager, &mut config)?;
        }
        cli::Commands::Profile(args) => {
            profile::handle_profile(args, &config_manager, &mut config)?;
        }
        cli::Commands::Config(args) => {
            config::handle_config(args.command, &config_manager, &mut config)?;
        }
    }

    config_manager.save(&config)?;
    Ok(())
}
