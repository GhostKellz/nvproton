//! Shader cache management for DXVK, vkd3d-proton, and NVIDIA
//!
//! Provides unified cache management with:
//! - Per-game cache isolation
//! - Cache size monitoring
//! - Cleanup utilities
//! - Cache import/export for sharing
//!
//! Note: Many functions here are reserved for future nvshader integration.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Cache types managed by nvproton
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    /// DXVK state cache (.dxvk-cache files)
    Dxvk,
    /// vkd3d-proton pipeline cache
    Vkd3d,
    /// NVIDIA GL shader cache
    NvidiaGl,
    /// Mesa/Vulkan pipeline cache
    Mesa,
    /// Steam shader pre-cache
    Steam,
}

impl CacheType {
    pub fn name(&self) -> &'static str {
        match self {
            CacheType::Dxvk => "dxvk",
            CacheType::Vkd3d => "vkd3d",
            CacheType::NvidiaGl => "nvidia-gl",
            CacheType::Mesa => "mesa",
            CacheType::Steam => "steam",
        }
    }

    pub fn env_var(&self) -> &'static str {
        match self {
            CacheType::Dxvk => "DXVK_STATE_CACHE_PATH",
            CacheType::Vkd3d => "VKD3D_SHADER_CACHE_PATH",
            CacheType::NvidiaGl => "__GL_SHADER_DISK_CACHE_PATH",
            CacheType::Mesa => "MESA_SHADER_CACHE_DIR",
            CacheType::Steam => "", // Not env-configurable
        }
    }
}

/// Cache directory structure
pub struct CachePaths {
    /// Base cache directory (~/.cache/nvproton)
    pub base: PathBuf,
    /// DXVK state cache directory
    pub dxvk: PathBuf,
    /// vkd3d-proton cache directory
    pub vkd3d: PathBuf,
    /// NVIDIA GL shader cache
    pub nvidia_gl: PathBuf,
    /// Mesa shader cache
    pub mesa: PathBuf,
}

impl CachePaths {
    /// Create cache paths with default locations
    pub fn new() -> Self {
        let base = dirs::cache_dir()
            .map(|d| d.join("nvproton"))
            .unwrap_or_else(|| PathBuf::from("/tmp/nvproton-cache"));

        Self {
            dxvk: base.join("dxvk"),
            vkd3d: base.join("vkd3d"),
            nvidia_gl: base.join("gl"),
            mesa: base.join("mesa"),
            base,
        }
    }

    /// Get cache path for a specific type
    pub fn get(&self, cache_type: CacheType) -> &Path {
        match cache_type {
            CacheType::Dxvk => &self.dxvk,
            CacheType::Vkd3d => &self.vkd3d,
            CacheType::NvidiaGl => &self.nvidia_gl,
            CacheType::Mesa => &self.mesa,
            CacheType::Steam => &self.base, // Steam manages its own
        }
    }

    /// Get game-specific cache path
    pub fn for_game(&self, cache_type: CacheType, game_id: &str) -> PathBuf {
        self.get(cache_type).join(game_id)
    }

    /// Ensure all cache directories exist
    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.base)
            .with_context(|| format!("Failed to create cache base dir: {:?}", self.base))?;
        fs::create_dir_all(&self.dxvk)?;
        fs::create_dir_all(&self.vkd3d)?;
        fs::create_dir_all(&self.nvidia_gl)?;
        fs::create_dir_all(&self.mesa)?;
        Ok(())
    }
}

impl Default for CachePaths {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache manager for shader caches
pub struct CacheManager {
    paths: CachePaths,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub cache_type: String,
    pub total_size_bytes: u64,
    pub file_count: usize,
    pub game_count: usize,
}

/// Per-game cache info
#[derive(Debug, Clone)]
pub struct GameCacheInfo {
    pub game_id: String,
    pub dxvk_size: u64,
    pub vkd3d_size: u64,
    pub gl_size: u64,
    pub total_size: u64,
    pub last_modified: Option<std::time::SystemTime>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let paths = CachePaths::new();
        paths.ensure()?;
        Ok(Self { paths })
    }

    /// Get cache paths
    pub fn paths(&self) -> &CachePaths {
        &self.paths
    }

    /// Set up cache paths for a game and return environment variables
    pub fn setup_for_game(&self, game_id: &str) -> Result<Vec<(String, String)>> {
        let mut env_vars = Vec::new();

        // DXVK
        let dxvk_path = self.paths.for_game(CacheType::Dxvk, game_id);
        fs::create_dir_all(&dxvk_path)?;
        env_vars.push((
            CacheType::Dxvk.env_var().to_string(),
            dxvk_path.to_string_lossy().to_string(),
        ));

        // vkd3d-proton
        let vkd3d_path = self.paths.for_game(CacheType::Vkd3d, game_id);
        fs::create_dir_all(&vkd3d_path)?;
        env_vars.push((
            CacheType::Vkd3d.env_var().to_string(),
            vkd3d_path.to_string_lossy().to_string(),
        ));

        // NVIDIA GL
        let gl_path = self.paths.for_game(CacheType::NvidiaGl, game_id);
        fs::create_dir_all(&gl_path)?;
        env_vars.push((
            CacheType::NvidiaGl.env_var().to_string(),
            gl_path.to_string_lossy().to_string(),
        ));

        // Mesa (shared, not per-game)
        env_vars.push((
            CacheType::Mesa.env_var().to_string(),
            self.paths.mesa.to_string_lossy().to_string(),
        ));

        Ok(env_vars)
    }

    /// Get cache statistics for all caches
    pub fn get_stats(&self) -> Result<Vec<CacheStats>> {
        let mut stats = Vec::new();

        for cache_type in [
            CacheType::Dxvk,
            CacheType::Vkd3d,
            CacheType::NvidiaGl,
            CacheType::Mesa,
        ] {
            let path = self.paths.get(cache_type);
            let (size, files, games) = Self::calculate_dir_stats(path)?;
            stats.push(CacheStats {
                cache_type: cache_type.name().to_string(),
                total_size_bytes: size,
                file_count: files,
                game_count: games,
            });
        }

        Ok(stats)
    }

    /// Get cache info for a specific game
    pub fn get_game_cache(&self, game_id: &str) -> Result<GameCacheInfo> {
        let dxvk_path = self.paths.for_game(CacheType::Dxvk, game_id);
        let vkd3d_path = self.paths.for_game(CacheType::Vkd3d, game_id);
        let gl_path = self.paths.for_game(CacheType::NvidiaGl, game_id);

        let dxvk_size = Self::dir_size(&dxvk_path).unwrap_or(0);
        let vkd3d_size = Self::dir_size(&vkd3d_path).unwrap_or(0);
        let gl_size = Self::dir_size(&gl_path).unwrap_or(0);

        let last_modified = Self::last_modified(&dxvk_path)
            .or_else(|| Self::last_modified(&vkd3d_path))
            .or_else(|| Self::last_modified(&gl_path));

        Ok(GameCacheInfo {
            game_id: game_id.to_string(),
            dxvk_size,
            vkd3d_size,
            gl_size,
            total_size: dxvk_size + vkd3d_size + gl_size,
            last_modified,
        })
    }

    /// List all games with caches
    pub fn list_games(&self) -> Result<Vec<String>> {
        let mut games = std::collections::HashSet::new();

        for cache_type in [CacheType::Dxvk, CacheType::Vkd3d, CacheType::NvidiaGl] {
            let path = self.paths.get(cache_type);
            if path.exists() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir()
                        && let Some(name) = entry.file_name().to_str()
                    {
                        games.insert(name.to_string());
                    }
                }
            }
        }

        let mut games: Vec<_> = games.into_iter().collect();
        games.sort();
        Ok(games)
    }

    /// Clear cache for a specific game
    pub fn clear_game(&self, game_id: &str) -> Result<u64> {
        let mut freed = 0u64;

        for cache_type in [CacheType::Dxvk, CacheType::Vkd3d, CacheType::NvidiaGl] {
            let path = self.paths.for_game(cache_type, game_id);
            if path.exists() {
                freed += Self::dir_size(&path).unwrap_or(0);
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove cache at {:?}", path))?;
            }
        }

        Ok(freed)
    }

    /// Clear all caches
    pub fn clear_all(&self) -> Result<u64> {
        let mut freed = 0u64;

        for cache_type in [
            CacheType::Dxvk,
            CacheType::Vkd3d,
            CacheType::NvidiaGl,
            CacheType::Mesa,
        ] {
            let path = self.paths.get(cache_type);
            if path.exists() {
                freed += Self::dir_size(path).unwrap_or(0);
                // Remove contents but keep directory
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        fs::remove_dir_all(entry.path())?;
                    } else {
                        fs::remove_file(entry.path())?;
                    }
                }
            }
        }

        Ok(freed)
    }

    /// Calculate total size and counts for a directory
    fn calculate_dir_stats(path: &Path) -> Result<(u64, usize, usize)> {
        if !path.exists() {
            return Ok((0, 0, 0));
        }

        let mut total_size = 0u64;
        let mut file_count = 0usize;
        let mut game_count = 0usize;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_type = entry.file_type()?;

            if entry_type.is_dir() {
                game_count += 1;
                let (size, files, _) = Self::calculate_dir_stats(&entry.path())?;
                total_size += size;
                file_count += files;
            } else if entry_type.is_file() {
                total_size += entry.metadata()?.len();
                file_count += 1;
            }
        }

        Ok((total_size, file_count, game_count))
    }

    /// Calculate size of a directory
    fn dir_size(path: &Path) -> Option<u64> {
        if !path.exists() {
            return None;
        }

        let mut size = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        size += metadata.len();
                    } else if metadata.is_dir() {
                        size += Self::dir_size(&entry.path()).unwrap_or(0);
                    }
                }
            }
        }
        Some(size)
    }

    /// Get last modified time of any file in directory
    fn last_modified(path: &Path) -> Option<std::time::SystemTime> {
        if !path.exists() {
            return None;
        }

        let mut latest: Option<std::time::SystemTime> = None;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata()
                    && let Ok(modified) = metadata.modified()
                {
                    latest = Some(match latest {
                        Some(l) if modified > l => modified,
                        Some(l) => l,
                        None => modified,
                    });
                }
            }
        }

        latest
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to create cache manager")
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_cache_type_names() {
        assert_eq!(CacheType::Dxvk.name(), "dxvk");
        assert_eq!(CacheType::Vkd3d.name(), "vkd3d");
    }
}
