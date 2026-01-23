//! MangoHud configuration generation for nvproton
//!
//! Generates MangoHud configuration files optimized for different scenarios.
//! Supports both global config and per-game overrides.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// MangoHud position on screen
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)] // Library API for config builders
pub enum Position {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
}

impl Position {
    #[allow(dead_code)] // Library API
    pub fn to_config(&self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::TopCenter => "top-center",
            Self::BottomCenter => "bottom-center",
        }
    }
}

/// MangoHud preset configurations
#[derive(Debug, Clone, Copy)]
pub enum MangoHudPreset {
    /// Minimal - FPS only
    Minimal,
    /// Compact - FPS + frametime graph
    Compact,
    /// Standard - FPS, frametime, GPU/CPU usage
    Standard,
    /// Full - All metrics including detailed hardware info
    Full,
    /// Steam Deck - Optimized for small screen
    SteamDeck,
    /// Competitive - Ultra minimal for esports
    Competitive,
    /// Debug - All debugging info
    Debug,
}

impl MangoHudPreset {
    #[allow(dead_code)] // Library API
    pub fn name(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Compact => "compact",
            Self::Standard => "standard",
            Self::Full => "full",
            Self::SteamDeck => "steam-deck",
            Self::Competitive => "competitive",
            Self::Debug => "debug",
        }
    }
}

/// MangoHud configuration builder
#[derive(Debug, Clone)]
pub struct MangoHudConfig {
    pub options: HashMap<String, String>,
}

impl Default for MangoHudConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MangoHudConfig {
    pub fn new() -> Self {
        Self {
            options: HashMap::new(),
        }
    }

    /// Create from a preset
    pub fn from_preset(preset: MangoHudPreset) -> Self {
        let mut config = Self::new();

        // Common settings
        config.set("legacy_layout", "false");
        config.set("round_corners", "8");
        config.set("font_size", "24");

        match preset {
            MangoHudPreset::Minimal => {
                config.set("fps", "");
                config.set("fps_only", "");
                config.set("position", "top-left");
            }

            MangoHudPreset::Compact => {
                config.set("fps", "");
                config.set("frametime", "");
                config.set("frame_timing", "");
                config.set("position", "top-left");
                config.set("font_size", "20");
                config.set("table_columns", "2");
            }

            MangoHudPreset::Standard => {
                config.set("fps", "");
                config.set("frametime", "");
                config.set("frame_timing", "");
                config.set("gpu_stats", "");
                config.set("gpu_temp", "");
                config.set("gpu_power", "");
                config.set("cpu_stats", "");
                config.set("cpu_temp", "");
                config.set("ram", "");
                config.set("vram", "");
                config.set("position", "top-left");
            }

            MangoHudPreset::Full => {
                config.set("full", "");
                config.set("position", "top-right");
                config.set("font_size", "20");
            }

            MangoHudPreset::SteamDeck => {
                config.set("fps", "");
                config.set("frametime", "");
                config.set("battery", "");
                config.set("battery_time", "");
                config.set("gpu_stats", "");
                config.set("gpu_temp", "");
                config.set("gpu_power", "");
                config.set("cpu_stats", "");
                config.set("cpu_temp", "");
                config.set("position", "top-left");
                config.set("font_size", "18"); // Smaller for 800p
                config.set("background_alpha", "0.5");
                config.set("round_corners", "4");
                config.set("table_columns", "2");
                // Steam Deck specific
                config.set("battery_icon", "");
                config.set("device_battery_icon", "");
            }

            MangoHudPreset::Competitive => {
                config.set("fps", "");
                config.set("fps_only", "");
                config.set("position", "top-left");
                config.set("font_size", "16");
                config.set("background_alpha", "0.3");
                config.set("no_display", ""); // Hidden by default, toggle with keybind
            }

            MangoHudPreset::Debug => {
                config.set("full", "");
                config.set("vulkan_driver", "");
                config.set("wine", "");
                config.set("engine_version", "");
                config.set("gpu_name", "");
                config.set("arch", "");
                config.set("position", "top-right");
                config.set("font_size", "18");
            }
        }

        config
    }

    /// Set a configuration option
    pub fn set(&mut self, key: &str, value: &str) -> &mut Self {
        self.options.insert(key.to_string(), value.to_string());
        self
    }

    // Builder methods below are part of the library API for programmatic config generation
    #[allow(dead_code)]
    /// Set position
    pub fn position(&mut self, pos: Position) -> &mut Self {
        self.set("position", pos.to_config())
    }

    #[allow(dead_code)]
    /// Enable FPS counter
    pub fn fps(&mut self) -> &mut Self {
        self.set("fps", "")
    }

    #[allow(dead_code)]
    /// Enable frametime graph
    pub fn frametime(&mut self) -> &mut Self {
        self.set("frametime", "");
        self.set("frame_timing", "")
    }

    #[allow(dead_code)]
    /// Enable GPU stats
    pub fn gpu_stats(&mut self) -> &mut Self {
        self.set("gpu_stats", "");
        self.set("gpu_temp", "");
        self.set("gpu_power", "")
    }

    #[allow(dead_code)]
    /// Enable CPU stats
    pub fn cpu_stats(&mut self) -> &mut Self {
        self.set("cpu_stats", "");
        self.set("cpu_temp", "")
    }

    #[allow(dead_code)]
    /// Enable battery stats (for portables)
    pub fn battery(&mut self) -> &mut Self {
        self.set("battery", "");
        self.set("battery_time", "")
    }

    #[allow(dead_code)]
    /// Set font size
    pub fn font_size(&mut self, size: u32) -> &mut Self {
        self.set("font_size", &size.to_string())
    }

    #[allow(dead_code)]
    /// Set background transparency
    pub fn background_alpha(&mut self, alpha: f32) -> &mut Self {
        self.set("background_alpha", &alpha.to_string())
    }

    #[allow(dead_code)]
    /// Set FPS limit
    pub fn fps_limit(&mut self, limit: u32) -> &mut Self {
        self.set("fps_limit", &limit.to_string())
    }

    #[allow(dead_code)]
    /// Set FPS limit method
    pub fn fps_limit_method(&mut self, method: &str) -> &mut Self {
        self.set("fps_limit_method", method)
    }

    #[allow(dead_code)]
    /// Enable toggle key
    pub fn toggle_fps_limit(&mut self, key: &str) -> &mut Self {
        self.set("toggle_fps_limit", key)
    }

    #[allow(dead_code)]
    /// Enable logging to file
    pub fn log_to_file(&mut self) -> &mut Self {
        self.set("output_folder", "/tmp/mangohud_logs");
        self.set("log_interval", "1000");
        self.set("autostart_log", "")
    }

    /// Render to string
    pub fn to_config_string(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# MangoHud configuration generated by nvproton".to_string());
        lines.push("# https://github.com/flightlessmango/MangoHud".to_string());
        lines.push(String::new());

        // Sort options for consistent output
        let mut sorted: Vec<_> = self.options.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in sorted {
            if value.is_empty() {
                lines.push(key.clone());
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }

        lines.join("\n")
    }

    /// Save to a file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {:?}", parent))?;
        }

        let content = self.to_config_string();
        let mut file = fs::File::create(path)
            .with_context(|| format!("failed to create MangoHud config at {:?}", path))?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

/// Get the MangoHud config directory
pub fn config_dir() -> Option<PathBuf> {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("MangoHud"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".config/MangoHud"));
    }
    None
}

/// Get per-game config path
pub fn game_config_path(game_name: &str) -> Option<PathBuf> {
    config_dir().map(|d| d.join(format!("{}.conf", game_name)))
}

/// Get global config path
pub fn global_config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("MangoHud.conf"))
}

/// Check if MangoHud is installed
pub fn is_installed() -> bool {
    // Check for mangohud binary
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            if PathBuf::from(dir).join("mangohud").exists() {
                return true;
            }
        }
    }

    // Check common locations
    let common_paths = ["/usr/bin/mangohud", "/usr/local/bin/mangohud"];
    for path in common_paths {
        if PathBuf::from(path).exists() {
            return true;
        }
    }

    false
}

/// Generate environment variables for MangoHud
pub fn env_vars(config: &MangoHudConfig) -> Vec<(String, String)> {
    let mut vars = Vec::new();

    vars.push(("MANGOHUD".to_string(), "1".to_string()));

    // Convert config options to MANGOHUD_CONFIG
    let config_str: Vec<String> = config
        .options
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                k.clone()
            } else {
                format!("{}={}", k, v)
            }
        })
        .collect();

    if !config_str.is_empty() {
        vars.push(("MANGOHUD_CONFIG".to_string(), config_str.join(",")));
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_preset() {
        let config = MangoHudConfig::from_preset(MangoHudPreset::Minimal);
        assert!(config.options.contains_key("fps"));
        assert!(config.options.contains_key("fps_only"));
    }

    #[test]
    fn test_config_string() {
        let mut config = MangoHudConfig::new();
        config.fps().frametime().position(Position::TopLeft);

        let output = config.to_config_string();
        assert!(output.contains("fps"));
        assert!(output.contains("frametime"));
        assert!(output.contains("position=top-left"));
    }

    #[test]
    fn test_steam_deck_preset() {
        let config = MangoHudConfig::from_preset(MangoHudPreset::SteamDeck);
        assert!(config.options.contains_key("battery"));
        assert!(config.options.contains_key("battery_time"));
    }
}
