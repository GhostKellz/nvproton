use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::ConfigPaths;
use crate::detection::steam::is_excluded_appid;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
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
                    profile: None,
                });
            entry.install_dir = game.install_dir.clone();
            entry.executable = game.executable.clone();
            entry.fingerprint = game.fingerprint.clone().or(entry.fingerprint.clone());
            entry.last_seen = timestamp;
            entry.metadata.extend(game.metadata.clone());
        }
    }

    /// Get a game by ID (searches all sources)
    pub fn get(&self, game_id: &str) -> Option<DetectedGame> {
        // Try direct key lookup first
        for (key, record) in &self.entries {
            if key.ends_with(&format!(":{}", game_id)) || key == game_id {
                return Some(record_to_detected(game_id, record));
            }
        }
        None
    }

    /// Iterate over all games (excluding Steam internals like Proton/Runtime)
    pub fn games(&self) -> impl Iterator<Item = DetectedGame> + '_ {
        self.entries.iter().filter_map(|(key, record)| {
            let id = key.split(':').nth(1).unwrap_or(key);
            // Skip excluded Steam apps (Proton, Runtime, Redistributables)
            if record.source == GameSource::Steam && is_excluded_appid(id) {
                return None;
            }
            Some(record_to_detected(id, record))
        })
    }

    /// Remove excluded Steam apps from database (cleanup)
    pub fn cleanup_excluded(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|key, record| {
            if record.source == GameSource::Steam {
                let id = key.split(':').nth(1).unwrap_or(key);
                !is_excluded_appid(id)
            } else {
                true
            }
        });
        before - self.entries.len()
    }

    /// Set profile for a game
    pub fn set_game_profile(&mut self, game_id: &str, profile: &str) {
        for (key, record) in &mut self.entries {
            if key.ends_with(&format!(":{}", game_id)) || key == game_id {
                record.profile = Some(profile.to_string());
                break;
            }
        }
    }

    /// Get profile for a game
    pub fn get_game_profile(&self, game_id: &str) -> Option<&str> {
        for (key, record) in &self.entries {
            if key.ends_with(&format!(":{}", game_id)) || key == game_id {
                return record.profile.as_deref();
            }
        }
        None
    }
}

fn game_key(game: &DetectedGame) -> String {
    format!("{}:{}", game.source, game.id)
}

fn record_to_detected(id: &str, record: &GameRecord) -> DetectedGame {
    DetectedGame {
        source: record.source.clone(),
        id: id.to_string(),
        name: record.name.clone(),
        install_dir: record.install_dir.clone(),
        executable: record.executable.clone(),
        fingerprint: record.fingerprint.clone(),
        metadata: record.metadata.clone(),
    }
}
