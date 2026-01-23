//! Proton-NV Detection Module
//!
//! Auto-detects Proton-NV installations and provides integration
//! with nvproton for optimized game launching.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Proton-NV installation information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtonNvInstallation {
    /// Installation path
    pub path: PathBuf,
    /// Version string (e.g., "Proton-NV-1.1-20260110")
    pub version: String,
    /// Full version info from version file
    pub version_info: Option<ProtonNvVersionInfo>,
    /// Whether this installation is valid
    pub valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtonNvVersionInfo {
    pub full_version: String,
    pub nvidia_driver_min: Option<String>,
    pub target_gpu: Option<String>,
}

/// Standard paths to search for Proton-NV installations
const PROTON_NV_SEARCH_PATHS: &[&str] = &[
    // Steam compatibility tools directory (installed via make install)
    ".local/share/Steam/compatibilitytools.d",
    // User-built installations
    ".local/share/proton-nv",
    // System-wide installations
    "/opt/proton-nv",
    "/usr/local/share/proton-nv",
];

/// Proton-NV detector
pub struct ProtonNvDetector {
    installations: Vec<ProtonNvInstallation>,
}

impl ProtonNvDetector {
    pub fn new() -> Self {
        Self {
            installations: Vec::new(),
        }
    }

    /// Scan for all Proton-NV installations
    pub fn scan(&mut self) -> Result<&[ProtonNvInstallation]> {
        self.installations.clear();

        let home = dirs::home_dir().unwrap_or_default();

        for search_path in PROTON_NV_SEARCH_PATHS {
            let path = if search_path.starts_with('/') {
                PathBuf::from(search_path)
            } else {
                home.join(search_path)
            };

            if path.exists() && path.is_dir() {
                self.scan_directory(&path)?;
            }
        }

        // Also check STEAM_COMPAT_TOOL_PATHS environment variable
        if let Ok(custom_paths) = std::env::var("STEAM_COMPAT_TOOL_PATHS") {
            for custom_path in custom_paths.split(':') {
                let path = PathBuf::from(custom_path);
                if path.exists() && path.is_dir() {
                    self.scan_directory(&path)?;
                }
            }
        }

        Ok(&self.installations)
    }

    /// Scan a directory for Proton-NV installations
    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if this looks like a Proton-NV installation
                if let Some(installation) = self.check_installation(&path) {
                    self.installations.push(installation);
                }
            }
        }

        Ok(())
    }

    /// Check if a directory contains a valid Proton-NV installation
    fn check_installation(&self, path: &Path) -> Option<ProtonNvInstallation> {
        let dir_name = path.file_name()?.to_string_lossy();

        // Must contain "proton-nv" or "Proton-NV" (case-insensitive)
        if !dir_name.to_lowercase().contains("proton-nv") {
            return None;
        }

        // Check for required files
        let proton_script = path.join("proton");
        let toolmanifest = path.join("toolmanifest.vdf");

        // At minimum, we need the proton script
        if !proton_script.exists() {
            return None;
        }

        // Read version info if available
        let version_info = self.read_version_info(path);
        let version = version_info
            .as_ref()
            .map(|v| v.full_version.clone())
            .unwrap_or_else(|| dir_name.to_string());

        Some(ProtonNvInstallation {
            path: path.to_path_buf(),
            version,
            version_info,
            valid: toolmanifest.exists(),
        })
    }

    /// Read version info from the version file
    fn read_version_info(&self, path: &Path) -> Option<ProtonNvVersionInfo> {
        let version_file = path.join("version");
        if !version_file.exists() {
            return None;
        }

        let content = fs::read_to_string(&version_file).ok()?;
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return None;
        }

        let full_version = lines[0].to_string();
        let nvidia_driver_min = lines.get(1).and_then(|l| {
            l.strip_prefix("NVIDIA Open ")
                .map(|s| s.trim_end_matches(" optimized").to_string())
        });
        let target_gpu = lines.get(2).and_then(|l| {
            l.strip_prefix("Target: ").map(|s| s.to_string())
        });

        Some(ProtonNvVersionInfo {
            full_version,
            nvidia_driver_min,
            target_gpu,
        })
    }

    /// Get the best (newest) Proton-NV installation
    pub fn get_best(&self) -> Option<&ProtonNvInstallation> {
        self.installations
            .iter()
            .filter(|i| i.valid)
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Get all detected installations
    #[allow(dead_code)] // Library API
    pub fn installations(&self) -> &[ProtonNvInstallation] {
        &self.installations
    }

    /// Check if Proton-NV is available
    #[allow(dead_code)] // Library API
    pub fn is_available(&self) -> bool {
        self.installations.iter().any(|i| i.valid)
    }
}

impl Default for ProtonNvDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variables to set when using Proton-NV
#[derive(Clone, Debug, Default)]
pub struct ProtonNvEnv {
    vars: Vec<(String, String)>,
}

impl ProtonNvEnv {
    /// Create environment setup for a Proton-NV installation
    pub fn from_installation(installation: &ProtonNvInstallation) -> Self {
        let mut env = Self::default();

        // Point to the Proton-NV installation
        env.set(
            "STEAM_COMPAT_DATA_PATH",
            installation.path.to_string_lossy(),
        );

        // Enable NVIDIA-specific optimizations
        env.set("PROTON_NV_ENABLED", "1");

        // Set profile environment variable for proton-NV internal use
        env.set("PROTON_NV_PROFILE", "gaming");

        // VK_NV_low_latency2 support
        env.set("DXVK_NVVK_ENABLE", "1");
        env.set("VKD3D_NVVK_ENABLE", "1");

        // Enable async shader compilation
        env.set("DXVK_ASYNC", "1");

        // NVIDIA Reflex support
        env.set("DXVK_NVAPI_ALLOW_REFLEX", "1");

        // Enable NVIDIA-specific NVAPI features
        env.set("DXVK_ENABLE_NVAPI", "1");

        // BAR1-aware allocation
        env.set("PROTON_NV_BAR1_AWARE", "1");

        env
    }

    /// Set an environment variable
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.push((key.into(), value.into()));
    }

    /// Get all environment variables
    pub fn vars(&self) -> &[(String, String)] {
        &self.vars
    }

    /// Convert to a HashMap
    #[allow(dead_code)] // Library API
    pub fn to_hashmap(&self) -> std::collections::HashMap<String, String> {
        self.vars.iter().cloned().collect()
    }
}

/// Quick check if Proton-NV is installed (without full scan)
#[allow(dead_code)] // Library API
pub fn is_proton_nv_installed() -> bool {
    let home = dirs::home_dir().unwrap_or_default();
    let steam_tools = home.join(".local/share/Steam/compatibilitytools.d");

    if steam_tools.exists() {
        if let Ok(entries) = fs::read_dir(&steam_tools) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("proton-nv") {
                    return true;
                }
            }
        }
    }

    false
}

/// Get the Proton-NV installation path if available
#[allow(dead_code)] // Library API
pub fn get_proton_nv_path() -> Option<PathBuf> {
    let mut detector = ProtonNvDetector::new();
    detector.scan().ok()?;
    detector.get_best().map(|i| i.path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = ProtonNvDetector::new();
        assert!(detector.installations().is_empty());
    }

    #[test]
    fn test_is_proton_nv_installed() {
        // This test just ensures the function doesn't panic
        let _ = is_proton_nv_installed();
    }

    #[test]
    fn test_env_setup() {
        let installation = ProtonNvInstallation {
            path: PathBuf::from("/test/path"),
            version: "test".into(),
            version_info: None,
            valid: true,
        };

        let env = ProtonNvEnv::from_installation(&installation);
        let vars = env.to_hashmap();

        assert_eq!(vars.get("PROTON_NV_ENABLED"), Some(&"1".to_string()));
        assert_eq!(vars.get("DXVK_ASYNC"), Some(&"1".to_string()));
    }
}
