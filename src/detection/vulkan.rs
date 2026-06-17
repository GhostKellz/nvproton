//! Vulkan capability detection using ash
//!
//! Detects Vulkan extensions relevant for vkd3d-proton and NVIDIA gaming:
//! - VK_EXT_descriptor_heap (DX12 descriptor mapping - 595+ driver)
//! - VK_EXT_descriptor_buffer (fallback path)
//! - VK_NV_raw_access_chains (existing optimization)
//! - VK_NV_low_latency2 (Reflex 2.0 - improved latency)
//! - VK_NV_extended_sparse_address_space (DX12 heap fix - 595+ driver)
//! - VK_EXT_present_timing (frame pacing - 595+ driver)

use anyhow::{Context, Result};
use ash::vk;
use std::ffi::CStr;

/// NVIDIA vendor ID
const NVIDIA_VENDOR_ID: u32 = 0x10DE;

/// First NVIDIA driver branch shipping the DX12 heap-fix feature set
/// (VK_EXT_descriptor_heap, VK_NV_extended_sparse_address_space, etc.).
/// Present in 595 and all later branches (600, 610, ...).
pub const DX12_HEAP_FIX_MIN_BRANCH: u32 = 595;

/// Known NVIDIA beta driver branch ranges `[start, end)`.
/// Branches not listed here are treated as stable/production
/// (e.g. 590, 600, 610 are production lines).
const BETA_BRANCH_RANGES: &[std::ops::Range<u32>] = &[
    580..590, // 580.x Vulkan beta series (first VK_EXT_descriptor_heap)
    595..600, // 595.x Vulkan beta series (DX12 heap fix)
];

/// Vulkan capabilities relevant for NVIDIA + vkd3d-proton
#[derive(Debug, Clone, Default)]
pub struct VulkanCapabilities {
    /// Driver version string (e.g., "595.45.04")
    pub driver_version: String,
    /// Major driver branch (e.g., 595)
    pub driver_branch: u32,
    /// VK_EXT_descriptor_heap support (main DX12 fix - 595+)
    pub descriptor_heap: bool,
    /// VK_EXT_descriptor_buffer support (fallback path)
    pub descriptor_buffer: bool,
    /// VK_NV_raw_access_chains support (existing optimization)
    pub raw_access_chains: bool,
    /// VK_NV_low_latency2 support (Reflex 2.0)
    pub low_latency2: bool,
    /// VK_NV_extended_sparse_address_space (DX12 heap fix - 595+)
    pub extended_sparse_address_space: bool,
    /// VK_EXT_present_timing support (frame pacing - 595+)
    pub present_timing: bool,
    /// GPU name
    pub gpu_name: String,
    /// Whether this is an NVIDIA GPU
    pub is_nvidia: bool,
}

impl VulkanCapabilities {
    /// Detect Vulkan capabilities for the primary NVIDIA GPU
    pub fn detect() -> Result<Self> {
        // Load Vulkan entry point
        let entry = unsafe { ash::Entry::load() }.context("Failed to load Vulkan library")?;

        // Create minimal instance without extensions
        let app_info = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 3, 0));

        let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("Failed to create Vulkan instance")?;

        // Enumerate physical devices
        let devices = unsafe { instance.enumerate_physical_devices() }
            .context("Failed to enumerate physical devices")?;

        let mut capabilities = VulkanCapabilities::default();

        for device in devices {
            let props = unsafe { instance.get_physical_device_properties(device) };

            // Check if this is an NVIDIA GPU
            if props.vendor_id == NVIDIA_VENDOR_ID {
                capabilities.is_nvidia = true;

                // Extract GPU name
                capabilities.gpu_name = unsafe {
                    CStr::from_ptr(props.device_name.as_ptr())
                        .to_string_lossy()
                        .into_owned()
                };

                // Decode NVIDIA driver version
                // NVIDIA uses a different encoding: major.minor.patch
                // Version format: (major << 22) | (minor << 14) | (patch << 6) | revision
                let version = props.driver_version;
                let major = (version >> 22) & 0x3FF;
                let minor = (version >> 14) & 0xFF;
                let patch = (version >> 6) & 0xFF;

                capabilities.driver_version = format!("{}.{}.{}", major, minor, patch);
                capabilities.driver_branch = major;

                // Enumerate device extensions
                let extensions = unsafe { instance.enumerate_device_extension_properties(device) }
                    .context("Failed to enumerate device extensions")?;

                for ext in extensions {
                    let name =
                        unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }.to_string_lossy();

                    match name.as_ref() {
                        "VK_EXT_descriptor_heap" => capabilities.descriptor_heap = true,
                        "VK_EXT_descriptor_buffer" => capabilities.descriptor_buffer = true,
                        "VK_NV_raw_access_chains" => capabilities.raw_access_chains = true,
                        "VK_NV_low_latency2" => capabilities.low_latency2 = true,
                        "VK_NV_extended_sparse_address_space" => {
                            capabilities.extended_sparse_address_space = true
                        }
                        "VK_EXT_present_timing" => capabilities.present_timing = true,
                        _ => {}
                    }
                }

                // Found NVIDIA GPU, stop searching
                break;
            }
        }

        // Cleanup
        unsafe { instance.destroy_instance(None) };

        if !capabilities.is_nvidia {
            anyhow::bail!("No NVIDIA GPU detected");
        }

        Ok(capabilities)
    }

    /// Check if VK_EXT_descriptor_heap is supported
    pub fn supports_descriptor_heap(&self) -> bool {
        self.descriptor_heap
    }

    /// Check if Reflex 2.0 (VK_NV_low_latency2) is supported
    pub fn supports_reflex2(&self) -> bool {
        self.low_latency2
    }

    /// Check if DX12 heap extension fix is available (595+)
    pub fn supports_dx12_heap_fix(&self) -> bool {
        self.extended_sparse_address_space
    }

    /// Check if this is a beta driver branch.
    /// Driven by `BETA_BRANCH_RANGES`; anything not listed is treated as
    /// stable/production (e.g. 590, 600, 610).
    pub fn is_beta_driver(&self) -> bool {
        BETA_BRANCH_RANGES
            .iter()
            .any(|range| range.contains(&self.driver_branch))
    }

    /// Whether the driver branch is new enough to ship the DX12 heap-fix
    /// feature set (595+). Extension probing is authoritative; this is the
    /// branch-based floor used for messaging and auto-enable fallbacks.
    pub fn has_dx12_heap_branch(&self) -> bool {
        self.driver_branch >= DX12_HEAP_FIX_MIN_BRANCH
    }

    /// Check if driver is expected to have descriptor_heap support
    /// Beta: 580.94.16+, 595.x+
    #[allow(dead_code)] // Library API for future driver version checks
    pub fn expected_descriptor_heap_support(&self) -> bool {
        // 595.x series has full support
        if self.driver_branch >= 595 {
            return true;
        }
        // 580.x beta branch: 580.94.16+ has descriptor_heap
        if self.driver_branch >= 580 && self.driver_branch < 590 {
            return self.parse_version_ge(580, 94, 16);
        }
        // Stable 590.x should have it
        self.driver_branch >= 590
    }

    /// Get a summary of the DX12 heap-fix driver features (595+).
    pub fn dx12_features(&self) -> Dx12DriverFeatures {
        Dx12DriverFeatures {
            descriptor_heap: self.descriptor_heap,
            extended_sparse: self.extended_sparse_address_space,
            low_latency2: self.low_latency2,
            present_timing: self.present_timing,
            raw_access_chains: self.raw_access_chains,
        }
    }

    /// Parse driver version and check if >= specified version
    fn parse_version_ge(&self, major: u32, minor: u32, patch: u32) -> bool {
        let parts: Vec<u32> = self
            .driver_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() < 3 {
            return false;
        }

        let (v_major, v_minor, v_patch) = (parts[0], parts[1], parts[2]);

        if v_major > major {
            return true;
        }
        if v_major < major {
            return false;
        }
        // v_major == major
        if v_minor > minor {
            return true;
        }
        if v_minor < minor {
            return false;
        }
        // v_minor == minor
        v_patch >= patch
    }
}

/// Summary of the DX12 heap-fix driver feature set (595+)
#[derive(Debug, Clone)]
#[allow(dead_code)] // Library API - fields used for feature tracking
pub struct Dx12DriverFeatures {
    pub descriptor_heap: bool,
    pub extended_sparse: bool,
    pub low_latency2: bool,
    pub present_timing: bool,
    pub raw_access_chains: bool,
}

impl Dx12DriverFeatures {
    /// Check if all major DX12 heap-fix features are available
    pub fn is_fully_supported(&self) -> bool {
        self.descriptor_heap && self.extended_sparse && self.low_latency2
    }

    /// Count of supported DX12 heap-fix features
    #[allow(dead_code)] // Library API
    pub fn feature_count(&self) -> usize {
        [
            self.descriptor_heap,
            self.extended_sparse,
            self.low_latency2,
            self.present_timing,
            self.raw_access_chains,
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

/// Display Vulkan capabilities summary
impl std::fmt::Display for VulkanCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Vulkan Capabilities:")?;
        writeln!(f, "  GPU: {}", self.gpu_name)?;
        write!(f, "  Driver: NVIDIA {}", self.driver_version)?;
        if self.is_beta_driver() {
            if self.has_dx12_heap_branch() {
                writeln!(f, " (beta - DX12 heap fixes)")?;
            } else {
                writeln!(f, " (beta)")?;
            }
        } else {
            writeln!(f)?;
        }

        writeln!(f, "\n  DX12/vkd3d-proton Extensions:")?;
        writeln!(
            f,
            "    VK_EXT_descriptor_heap: {}",
            if self.descriptor_heap {
                "supported"
            } else {
                "not available"
            }
        )?;
        writeln!(
            f,
            "    VK_EXT_descriptor_buffer: {}",
            if self.descriptor_buffer {
                "supported"
            } else {
                "not available"
            }
        )?;
        writeln!(
            f,
            "    VK_NV_extended_sparse_address_space: {}",
            if self.extended_sparse_address_space {
                "supported (DX12 heap fix)"
            } else {
                "not available"
            }
        )?;
        writeln!(
            f,
            "    VK_NV_raw_access_chains: {}",
            if self.raw_access_chains {
                "supported"
            } else {
                "not available"
            }
        )?;

        writeln!(f, "\n  Gaming/Latency Extensions:")?;
        writeln!(
            f,
            "    VK_NV_low_latency2: {}",
            if self.low_latency2 {
                "supported (Reflex 2.0)"
            } else {
                "not available"
            }
        )?;
        writeln!(
            f,
            "    VK_EXT_present_timing: {}",
            if self.present_timing {
                "supported"
            } else {
                "not available"
            }
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let mut caps = VulkanCapabilities {
            driver_version: "580.94.16".into(),
            ..Default::default()
        };

        // Test exact version
        assert!(caps.parse_version_ge(580, 94, 16));
        assert!(!caps.parse_version_ge(580, 94, 17));

        // Test higher patch
        caps.driver_version = "580.94.20".into();
        assert!(caps.parse_version_ge(580, 94, 16));

        // Test higher minor
        caps.driver_version = "580.95.0".into();
        assert!(caps.parse_version_ge(580, 94, 16));

        // Test higher major
        caps.driver_version = "590.0.0".into();
        assert!(caps.parse_version_ge(580, 94, 16));

        // Test lower version
        caps.driver_version = "580.94.10".into();
        assert!(!caps.parse_version_ge(580, 94, 16));
    }

    #[test]
    fn test_beta_driver_detection() {
        let mut caps = VulkanCapabilities {
            driver_branch: 580,
            ..Default::default()
        };

        // 580.x beta series
        assert!(caps.is_beta_driver());

        caps.driver_branch = 585;
        assert!(caps.is_beta_driver());

        // 590.x stable
        caps.driver_branch = 590;
        assert!(!caps.is_beta_driver());

        // 595.x beta series (first DX12 heap-fix branch)
        caps.driver_branch = 595;
        assert!(caps.is_beta_driver());
        assert!(caps.has_dx12_heap_branch());

        caps.driver_branch = 599;
        assert!(caps.is_beta_driver());
        assert!(caps.has_dx12_heap_branch());

        // Pre-580 should not be beta and lacks the DX12 heap-fix set
        caps.driver_branch = 575;
        assert!(!caps.is_beta_driver());
        assert!(!caps.has_dx12_heap_branch());

        // 600/610 are stable production branches that still ship DX12 heap fixes
        caps.driver_branch = 600;
        assert!(!caps.is_beta_driver());
        assert!(caps.has_dx12_heap_branch());

        caps.driver_branch = 610;
        assert!(!caps.is_beta_driver());
        assert!(caps.has_dx12_heap_branch());
    }

    #[test]
    fn test_dx12_features() {
        let mut caps = VulkanCapabilities {
            descriptor_heap: true,
            extended_sparse_address_space: true,
            low_latency2: true,
            present_timing: true,
            raw_access_chains: true,
            ..Default::default()
        };

        let features = caps.dx12_features();
        assert!(features.is_fully_supported());
        assert_eq!(features.feature_count(), 5);

        caps.low_latency2 = false;
        let features = caps.dx12_features();
        assert!(!features.is_fully_supported());
        assert_eq!(features.feature_count(), 4);
    }
}
