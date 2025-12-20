use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::fingerprint;
use super::{DetectedGame, DetectionContext, GameSource};

pub struct LutrisDetector;

impl LutrisDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(
        &self,
        ctx: &DetectionContext<'_>,
        include_fingerprint: bool,
    ) -> Result<Vec<DetectedGame>> {
        let lutris_root = match ctx.config.library_paths.lutris.as_ref() {
            Some(path) => path.clone(),
            None => return Ok(Vec::new()),
        };
        let db_path = lutris_root.join("pga.db");
        if !db_path.exists() {
            return Ok(Vec::new());
        }
        let connection = Connection::open(&db_path)
            .with_context(|| format!("failed to open lutris database at {:?}", db_path))?;

        // Try new schema first, then fall back to old schema
        let query_result = connection
            .prepare("SELECT slug, name, directory, runner FROM games")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {
                    Ok(LutrisGame {
                        slug: row.get(0)?,
                        name: row.get(1)?,
                        directory: row.get(2)?,
                        executable: None, // New schema doesn't have exe column
                        runner: row.get::<_, Option<String>>(3)?,
                    })
                })
                .map(|rows| rows.collect::<Result<Vec<_>, _>>())
            });

        let lutris_games: Vec<LutrisGame> = match query_result {
            Ok(Ok(games)) => games,
            _ => {
                // Fall back to old schema with exe column
                let mut statement =
                    connection.prepare("SELECT slug, name, directory, exe, runner FROM games")?;
                statement
                    .query_map([], |row| {
                        Ok(LutrisGame {
                            slug: row.get(0)?,
                            name: row.get(1)?,
                            directory: row.get(2)?,
                            executable: row.get::<_, Option<String>>(3)?,
                            runner: row.get::<_, Option<String>>(4)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?
            }
        };
        let mut games = Vec::new();
        for entry in lutris_games {
            let install_dir = PathBuf::from(&entry.directory);
            let executable_path = entry.executable.as_ref().map(|exe| install_dir.join(exe));
            let fingerprint_value = if include_fingerprint {
                executable_path
                    .as_ref()
                    .and_then(|exe| fingerprint::fingerprint_file(exe).ok())
            } else {
                None
            };
            let mut metadata = HashMap::new();
            if let Some(runner) = entry.runner.clone() {
                metadata.insert("runner".into(), runner);
            }
            games.push(DetectedGame {
                source: GameSource::Lutris,
                id: entry.slug.clone(),
                name: entry.name.clone(),
                install_dir,
                executable: executable_path,
                fingerprint: fingerprint_value,
                metadata,
            });
        }
        Ok(games)
    }
}

struct LutrisGame {
    slug: String,
    name: String,
    directory: String,
    executable: Option<String>,
    runner: Option<String>,
}
