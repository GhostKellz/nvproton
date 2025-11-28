mod database;
pub mod fingerprint;
mod heroic;
mod lutris;
mod steam;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::{DetectArgs, DetectCommand, OutputFormat};
use crate::config::{ConfigManager, NvConfig};

pub use database::GameDatabase;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedGame {
    pub source: GameSource,
    pub id: String,
    pub name: String,
    pub install_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameSource {
    Steam,
    Heroic,
    Lutris,
    Unknown,
}

impl fmt::Display for GameSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameSource::Steam => write!(f, "steam"),
            GameSource::Heroic => write!(f, "heroic"),
            GameSource::Lutris => write!(f, "lutris"),
            GameSource::Unknown => write!(f, "unknown"),
        }
    }
}

pub struct DetectionContext<'a> {
    pub config: &'a NvConfig,
    pub manager: &'a ConfigManager,
}

impl<'a> DetectionContext<'a> {
    pub fn new(config: &'a NvConfig, manager: &'a ConfigManager) -> Self {
        Self { config, manager }
    }
}

pub fn handle_detect(
    args: DetectArgs,
    manager: &ConfigManager,
    config: &mut NvConfig,
) -> Result<()> {
    let ctx = DetectionContext::new(config, manager);
    match args.command {
        DetectCommand::Steam(opts) => {
            let games = steam::SteamDetector::new().detect(&ctx, opts.fingerprint)?;
            output_games(&games, opts.format);
            maybe_update_database(&ctx, opts.update_db, &games)?;
        }
        DetectCommand::Heroic(opts) => {
            let games = heroic::HeroicDetector::new().detect(&ctx, opts.fingerprint)?;
            output_games(&games, opts.format);
            maybe_update_database(&ctx, opts.update_db, &games)?;
        }
        DetectCommand::Lutris(opts) => {
            let games = lutris::LutrisDetector::new().detect(&ctx, opts.fingerprint)?;
            output_games(&games, opts.format);
            maybe_update_database(&ctx, opts.update_db, &games)?;
        }
        DetectCommand::All(opts) => {
            let mut all_games = Vec::new();
            all_games.extend(steam::SteamDetector::new().detect(&ctx, opts.fingerprint)?);
            all_games.extend(heroic::HeroicDetector::new().detect(&ctx, opts.fingerprint)?);
            all_games.extend(lutris::LutrisDetector::new().detect(&ctx, opts.fingerprint)?);
            output_games(&all_games, opts.format);
            maybe_update_database(&ctx, opts.update_db, &all_games)?;
        }
    }
    Ok(())
}

fn maybe_update_database(
    ctx: &DetectionContext<'_>,
    update: bool,
    games: &[DetectedGame],
) -> Result<()> {
    if !update {
        return Ok(());
    }
    let mut db = GameDatabase::load_or_default(ctx.manager.paths())?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    db.merge_detected(games, timestamp);
    db.save(ctx.manager.paths())
}

fn output_games(games: &[DetectedGame], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for game in games {
                println!(
                    "[{source}] {name} ({id})\n  install: {install:?}\n  executable: {exe:?}\n  fingerprint: {finger:?}\n",
                    source = game.source,
                    name = game.name,
                    id = game.id,
                    install = game.install_dir,
                    exe = game.executable,
                    finger = game.fingerprint
                );
            }
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(games) {
                println!("{}", json);
            }
        }
        OutputFormat::Yaml => {
            if let Ok(yaml) = serde_yaml::to_string(games) {
                println!("{}", yaml);
            }
        }
    }
}
