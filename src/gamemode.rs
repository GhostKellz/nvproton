//! Feral GameMode integration for nvproton
//!
//! Provides integration with Feral Interactive's GameMode daemon
//! for optimized gaming performance on Linux.
//!
//! Features:
//! - CPU governor switching (performance/powersave)
//! - GPU performance mode
//! - Process nice value adjustment
//! - I/O priority optimization
//! - Custom scripts execution

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// GameMode configuration
#[derive(Debug, Clone)]
pub struct GameModeConfig {
    pub general: GeneralConfig,
    pub gpu: GpuConfig,
    pub cpu: CpuConfig,
    pub custom: CustomConfig,
}

impl Default for GameModeConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            gpu: GpuConfig::default(),
            cpu: CpuConfig::default(),
            custom: CustomConfig::default(),
        }
    }
}

/// General GameMode settings
#[derive(Debug, Clone)]
pub struct GeneralConfig {
    /// Process renice value (-20 to 19, lower = higher priority)
    pub renice: i32,
    /// I/O priority class (none, realtime, best-effort, idle)
    pub ioprio: String,
    /// Soft realtime scheduling
    pub softrealtime: bool,
    /// Inhibit screensaver
    pub inhibit_screensaver: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            renice: 0,
            ioprio: "best-effort".to_string(),
            softrealtime: false,
            inhibit_screensaver: true,
        }
    }
}

/// GPU-specific settings
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Apply GPU optimizations
    pub apply_gpu_optimizations: bool,
    /// GPU device index (0 for first GPU)
    pub gpu_device: i32,
    /// NVIDIA performance level (0-3, where 3 is max performance)
    pub nv_perf_level: i32,
    /// NVIDIA power mode (0 = adaptive, 1 = prefer max performance)
    pub nv_powermizer_mode: i32,
    /// AMD GPU performance level (auto, low, high, manual)
    pub amd_performance_level: String,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            apply_gpu_optimizations: true,
            gpu_device: 0,
            nv_perf_level: 3,
            nv_powermizer_mode: 1,
            amd_performance_level: "high".to_string(),
        }
    }
}

/// CPU-specific settings
#[derive(Debug, Clone)]
pub struct CpuConfig {
    /// CPU governor when gaming
    pub governor: String,
    /// Park cores when not gaming
    pub park_cores: bool,
    /// Pin game to specific cores
    pub pin_cores: bool,
    /// Core affinity mask (empty = all cores)
    pub core_affinity: String,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            governor: "performance".to_string(),
            park_cores: false,
            pin_cores: false,
            core_affinity: String::new(),
        }
    }
}

/// Custom script settings
#[derive(Debug, Clone, Default)]
pub struct CustomConfig {
    /// Script to run when game starts
    pub start_script: Option<String>,
    /// Script to run when game ends
    pub end_script: Option<String>,
}

impl GameModeConfig {
    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            general: GeneralConfig {
                renice: -10,
                ioprio: "best-effort".to_string(),
                softrealtime: true,
                inhibit_screensaver: true,
            },
            gpu: GpuConfig {
                apply_gpu_optimizations: true,
                gpu_device: 0,
                nv_perf_level: 3,
                nv_powermizer_mode: 1,
                amd_performance_level: "high".to_string(),
            },
            cpu: CpuConfig {
                governor: "performance".to_string(),
                park_cores: false,
                pin_cores: false,
                core_affinity: String::new(),
            },
            custom: CustomConfig::default(),
        }
    }

    /// Create a battery-saving configuration
    pub fn power_save() -> Self {
        Self {
            general: GeneralConfig {
                renice: 0,
                ioprio: "best-effort".to_string(),
                softrealtime: false,
                inhibit_screensaver: true,
            },
            gpu: GpuConfig {
                apply_gpu_optimizations: true,
                gpu_device: 0,
                nv_perf_level: 1,
                nv_powermizer_mode: 0,
                amd_performance_level: "auto".to_string(),
            },
            cpu: CpuConfig {
                governor: "powersave".to_string(),
                park_cores: true,
                pin_cores: false,
                core_affinity: String::new(),
            },
            custom: CustomConfig::default(),
        }
    }

    /// Create competitive gaming configuration (lowest latency)
    pub fn competitive() -> Self {
        Self {
            general: GeneralConfig {
                renice: -15,
                ioprio: "realtime".to_string(),
                softrealtime: true,
                inhibit_screensaver: true,
            },
            gpu: GpuConfig {
                apply_gpu_optimizations: true,
                gpu_device: 0,
                nv_perf_level: 3,
                nv_powermizer_mode: 1,
                amd_performance_level: "high".to_string(),
            },
            cpu: CpuConfig {
                governor: "performance".to_string(),
                park_cores: false,
                pin_cores: true, // Pin to P-cores on hybrid CPUs
                core_affinity: String::new(),
            },
            custom: CustomConfig::default(),
        }
    }

    /// Render to INI format string
    pub fn to_ini_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push("; GameMode configuration generated by nvproton".to_string());
        lines.push("; https://github.com/FeralInteractive/gamemode".to_string());
        lines.push(String::new());

        // General section
        lines.push("[general]".to_string());
        lines.push(format!("renice={}", self.general.renice));
        lines.push(format!("ioprio={}", self.general.ioprio));
        lines.push(format!(
            "softrealtime={}",
            if self.general.softrealtime { "on" } else { "off" }
        ));
        lines.push(format!(
            "inhibit_screensaver={}",
            self.general.inhibit_screensaver as i32
        ));
        lines.push(String::new());

        // GPU section
        lines.push("[gpu]".to_string());
        lines.push(format!(
            "apply_gpu_optimisations={}",
            if self.gpu.apply_gpu_optimizations {
                "accept-responsibility"
            } else {
                "0"
            }
        ));
        lines.push(format!("gpu_device={}", self.gpu.gpu_device));
        lines.push(format!("nv_perf_level={}", self.gpu.nv_perf_level));
        lines.push(format!("nv_powermizer_mode={}", self.gpu.nv_powermizer_mode));
        lines.push(format!(
            "amd_performance_level={}",
            self.gpu.amd_performance_level
        ));
        lines.push(String::new());

        // CPU section
        lines.push("[cpu]".to_string());
        lines.push(format!("desiredgov={}", self.cpu.governor));
        lines.push(format!(
            "park_cores={}",
            if self.cpu.park_cores { "yes" } else { "no" }
        ));
        if self.cpu.pin_cores && !self.cpu.core_affinity.is_empty() {
            lines.push(format!("pin_cores={}", self.cpu.core_affinity));
        }
        lines.push(String::new());

        // Custom section
        if self.custom.start_script.is_some() || self.custom.end_script.is_some() {
            lines.push("[custom]".to_string());
            if let Some(ref script) = self.custom.start_script {
                lines.push(format!("start={}", script));
            }
            if let Some(ref script) = self.custom.end_script {
                lines.push(format!("end={}", script));
            }
        }

        lines.join("\n")
    }

    /// Save to config file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {:?}", parent))?;
        }

        let content = self.to_ini_string();
        let mut file = fs::File::create(path)
            .with_context(|| format!("failed to create GameMode config at {:?}", path))?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

/// Check if GameMode daemon is installed
pub fn is_installed() -> bool {
    // Check for gamemoded binary
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            if PathBuf::from(dir).join("gamemoded").exists() {
                return true;
            }
        }
    }

    // Check common locations
    let common_paths = ["/usr/bin/gamemoded", "/usr/local/bin/gamemoded"];
    for path in common_paths {
        if PathBuf::from(path).exists() {
            return true;
        }
    }

    false
}

/// Check if GameMode daemon is running
pub fn is_running() -> bool {
    Command::new("gamemoded")
        .arg("-s")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get GameMode config directory
pub fn config_dir() -> Option<PathBuf> {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("gamemode"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".config/gamemode"));
    }
    None
}

/// Get global config path
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("gamemode.ini"))
}

/// Generate environment variables for enabling GameMode
pub fn env_vars() -> Vec<(String, String)> {
    vec![("GAMEMODERUNEXEC".to_string(), "gamemoderun".to_string())]
}

/// Generate launch prefix for running with GameMode
pub fn launch_prefix() -> &'static str {
    "gamemoderun"
}

/// Request game mode for the current process (returns registration ID)
pub fn request_start() -> Result<()> {
    let status = Command::new("gamemoded")
        .arg("-r")
        .status()
        .context("failed to request gamemode")?;

    if !status.success() {
        anyhow::bail!("gamemode request failed");
    }
    Ok(())
}

/// End game mode for the current process
pub fn request_end() -> Result<()> {
    let status = Command::new("gamemoded")
        .arg("-u")
        .status()
        .context("failed to end gamemode")?;

    if !status.success() {
        anyhow::bail!("gamemode end failed");
    }
    Ok(())
}

/// Get current GameMode status
pub fn status() -> Result<GameModeStatus> {
    let output = Command::new("gamemoded")
        .arg("-s")
        .output()
        .context("failed to query gamemode status")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(GameModeStatus {
        running: output.status.success(),
        client_count: parse_client_count(&stdout),
    })
}

/// GameMode daemon status
#[derive(Debug)]
pub struct GameModeStatus {
    pub running: bool,
    pub client_count: u32,
}

fn parse_client_count(output: &str) -> u32 {
    // Parse output like "gamemode is active with 2 clients"
    output
        .split_whitespace()
        .find_map(|word| word.parse::<u32>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameModeConfig::default();
        assert_eq!(config.general.renice, 0);
        assert_eq!(config.cpu.governor, "performance");
    }

    #[test]
    fn test_high_performance_config() {
        let config = GameModeConfig::high_performance();
        assert_eq!(config.general.renice, -10);
        assert!(config.general.softrealtime);
    }

    #[test]
    fn test_competitive_config() {
        let config = GameModeConfig::competitive();
        assert_eq!(config.general.renice, -15);
        assert!(config.cpu.pin_cores);
    }

    #[test]
    fn test_ini_output() {
        let config = GameModeConfig::default();
        let ini = config.to_ini_string();
        assert!(ini.contains("[general]"));
        assert!(ini.contains("[gpu]"));
        assert!(ini.contains("[cpu]"));
    }
}
