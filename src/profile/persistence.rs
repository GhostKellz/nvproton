//! SQLite-backed game-to-profile persistence
//!
//! Stores associations between games and their assigned profiles,
//! enabling automatic profile loading when games are launched.

use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

/// Game-to-profile binding record
#[derive(Debug, Clone)]
#[allow(dead_code)] // Library API for game-profile persistence
pub struct ProfileBinding {
    /// Game ID (e.g., Steam app ID or launcher-specific identifier)
    pub game_id: String,
    /// Profile name
    pub profile_name: String,
    /// When binding was created (Unix timestamp)
    pub created_at: i64,
    /// When binding was last updated (Unix timestamp)
    pub updated_at: i64,
}

/// SQLite-backed profile persistence manager
pub struct ProfilePersistence {
    conn: Connection,
}

impl ProfilePersistence {
    /// Open or create the profile database at the given path
    pub fn open(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create database directory: {:?}", parent))?;
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("failed to open profile database at {:?}", db_path))?;

        let persistence = Self { conn };
        persistence.init_schema()?;
        Ok(persistence)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS profile_bindings (
                game_id TEXT PRIMARY KEY NOT NULL,
                profile_name TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_profile_name
                ON profile_bindings(profile_name);
        ",
            )
            .context("failed to initialize profile database schema")?;
        Ok(())
    }

    /// Bind a game to a profile
    pub fn bind(&self, game_id: &str, profile_name: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO profile_bindings (game_id, profile_name)
                 VALUES (?1, ?2)
                 ON CONFLICT(game_id) DO UPDATE SET
                     profile_name = excluded.profile_name,
                     updated_at = strftime('%s', 'now')",
                params![game_id, profile_name],
            )
            .with_context(|| format!("failed to bind game '{}' to profile '{}'", game_id, profile_name))?;
        Ok(())
    }

    /// Remove a game's profile binding
    #[allow(dead_code)] // Library API
    pub fn unbind(&self, game_id: &str) -> Result<bool> {
        let count = self
            .conn
            .execute(
                "DELETE FROM profile_bindings WHERE game_id = ?1",
                params![game_id],
            )
            .with_context(|| format!("failed to unbind game '{}'", game_id))?;
        Ok(count > 0)
    }

    /// Get the profile bound to a game
    pub fn get_binding(&self, game_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT profile_name FROM profile_bindings WHERE game_id = ?1")
            .context("failed to prepare binding query")?;

        let result = stmt
            .query_row(params![game_id], |row| row.get(0))
            .optional()
            .context("failed to query binding")?;

        Ok(result)
    }

    /// Get full binding record for a game
    #[allow(dead_code)] // Library API
    pub fn get_binding_record(&self, game_id: &str) -> Result<Option<ProfileBinding>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT game_id, profile_name, created_at, updated_at
                 FROM profile_bindings WHERE game_id = ?1",
            )
            .context("failed to prepare binding query")?;

        let result = stmt
            .query_row(params![game_id], |row| {
                Ok(ProfileBinding {
                    game_id: row.get(0)?,
                    profile_name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .optional()
            .context("failed to query binding record")?;

        Ok(result)
    }

    /// List all games bound to a specific profile
    #[allow(dead_code)] // Library API
    pub fn games_with_profile(&self, profile_name: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_id FROM profile_bindings WHERE profile_name = ?1 ORDER BY game_id")
            .context("failed to prepare games query")?;

        let games = stmt
            .query_map(params![profile_name], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .context("failed to collect game IDs")?;

        Ok(games)
    }

    /// List all bindings
    #[allow(dead_code)] // Library API
    pub fn list_bindings(&self) -> Result<Vec<ProfileBinding>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT game_id, profile_name, created_at, updated_at
                 FROM profile_bindings ORDER BY game_id",
            )
            .context("failed to prepare list query")?;

        let bindings = stmt
            .query_map([], |row| {
                Ok(ProfileBinding {
                    game_id: row.get(0)?,
                    profile_name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to collect bindings")?;

        Ok(bindings)
    }

    /// Remove all bindings for a profile (e.g., when profile is deleted)
    #[allow(dead_code)] // Library API
    pub fn unbind_profile(&self, profile_name: &str) -> Result<usize> {
        let count = self
            .conn
            .execute(
                "DELETE FROM profile_bindings WHERE profile_name = ?1",
                params![profile_name],
            )
            .with_context(|| format!("failed to unbind profile '{}'", profile_name))?;
        Ok(count)
    }

    /// Count total bindings
    #[allow(dead_code)] // Library API
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM profile_bindings", [], |row| row.get(0))
            .context("failed to count bindings")?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        (dir, path)
    }

    #[test]
    fn test_bind_and_get() {
        let (_dir, path) = temp_db();
        let persistence = ProfilePersistence::open(&path).unwrap();

        persistence.bind("steam:123456", "high-quality").unwrap();
        let result = persistence.get_binding("steam:123456").unwrap();
        assert_eq!(result, Some("high-quality".to_string()));
    }

    #[test]
    fn test_unbind() {
        let (_dir, path) = temp_db();
        let persistence = ProfilePersistence::open(&path).unwrap();

        persistence.bind("steam:123456", "high-quality").unwrap();
        assert!(persistence.unbind("steam:123456").unwrap());
        assert!(persistence.get_binding("steam:123456").unwrap().is_none());
    }

    #[test]
    fn test_update_binding() {
        let (_dir, path) = temp_db();
        let persistence = ProfilePersistence::open(&path).unwrap();

        persistence.bind("steam:123456", "high-quality").unwrap();
        persistence.bind("steam:123456", "low-latency").unwrap();

        let result = persistence.get_binding("steam:123456").unwrap();
        assert_eq!(result, Some("low-latency".to_string()));

        // Should still be just one binding
        assert_eq!(persistence.count().unwrap(), 1);
    }

    #[test]
    fn test_games_with_profile() {
        let (_dir, path) = temp_db();
        let persistence = ProfilePersistence::open(&path).unwrap();

        persistence.bind("steam:111", "performance").unwrap();
        persistence.bind("steam:222", "performance").unwrap();
        persistence.bind("steam:333", "quality").unwrap();

        let games = persistence.games_with_profile("performance").unwrap();
        assert_eq!(games.len(), 2);
        assert!(games.contains(&"steam:111".to_string()));
        assert!(games.contains(&"steam:222".to_string()));
    }

    #[test]
    fn test_list_bindings() {
        let (_dir, path) = temp_db();
        let persistence = ProfilePersistence::open(&path).unwrap();

        persistence.bind("steam:111", "performance").unwrap();
        persistence.bind("steam:222", "quality").unwrap();

        let bindings = persistence.list_bindings().unwrap();
        assert_eq!(bindings.len(), 2);
    }
}
