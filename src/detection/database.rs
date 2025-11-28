use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::ConfigPaths;
use crate::detection::{DetectedGame, GameSource};

const DATABASE_FILE: &str = "games.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameDatabase {
    #[serde(default)]
    pub entries: HashMap<String, GameRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub source: GameSource,
    pub name: String,
    pub install_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    pub last_seen: u64,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl GameDatabase {
    pub fn load_or_default(paths: &ConfigPaths) -> Result<Self> {
        let db_path = paths.games_dir.join(DATABASE_FILE);
        if !db_path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&db_path)
            .with_context(|| format!("failed to read game database at {:?}", db_path))?;
        let db: GameDatabase =
            serde_yaml::from_str(&contents).context("failed to parse game database YAML")?;
        Ok(db)
    }

    pub fn save(&self, paths: &ConfigPaths) -> Result<()> {
        let db_path = paths.games_dir.join(DATABASE_FILE);
        fs::create_dir_all(&paths.games_dir).with_context(|| {
            format!("failed to create games directory at {:?}", paths.games_dir)
        })?;
        let encoded = serde_yaml::to_string(self).context("failed to serialize game database")?;
        fs::write(&db_path, encoded)
            .with_context(|| format!("failed to write game database at {:?}", db_path))?;
        Ok(())
    }

    pub fn merge_detected(&mut self, games: &[DetectedGame], timestamp: u64) {
        for game in games {
            let entry = self
                .entries
                .entry(game_key(game))
                .or_insert_with(|| GameRecord {
                    source: game.source.clone(),
                    name: game.name.clone(),
                    install_dir: game.install_dir.clone(),
                    executable: game.executable.clone(),
                    fingerprint: game.fingerprint.clone(),
                    last_seen: timestamp,
                    metadata: game.metadata.clone(),
                });
            entry.install_dir = game.install_dir.clone();
            entry.executable = game.executable.clone();
            entry.fingerprint = game.fingerprint.clone().or(entry.fingerprint.clone());
            entry.last_seen = timestamp;
            entry.metadata.extend(game.metadata.clone());
        }
    }
}

fn game_key(game: &DetectedGame) -> String {
    format!("{}:{}", game.source, game.id)
}
