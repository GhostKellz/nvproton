//! System status and driver readiness reporting
//!
//! Provides comprehensive system status including:
//! - Vulkan driver and extension support (595+ features)
//! - vkd3d-proton installation and version
//! - Proton-NV detection
//! - DX12 readiness (descriptor_heap + extended sparse support)
//! - Reflex 2.0 and frame pacing capabilities

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cli::{OutputFormat, StatusArgs};
use crate::config::ConfigManager;
use crate::detection::proton_nv::ProtonNvDetector;
use crate::detection::VulkanCapabilities;
use crate::gamemode;
use crate::mangohud;

/// Comprehensive system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub vulkan: Option<VulkanStatus>,
    pub vkd3d_proton: Option<Vkd3dProtonStatus>,
    pub proton_nv: Option<ProtonNvStatus>,
    pub tools: ToolsStatus,
    pub dx12_ready: bool,
    pub dx12_ready_reason: String,
}

/// Vulkan driver and extension status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulkanStatus {
    pub gpu_name: String,
    pub driver_version: String,
    pub driver_branch: u32,
    pub is_beta: bool,
    pub is_595_series: bool,
    // DX12/vkd3d-proton extensions
    pub descriptor_heap: bool,
    pub descriptor_buffer: bool,
    pub raw_access_chains: bool,
    pub extended_sparse_address_space: bool,
    // Gaming/latency extensions
    pub low_latency2: bool,
    pub present_timing: bool,
}

impl From<&VulkanCapabilities> for VulkanStatus {
    fn from(caps: &VulkanCapabilities) -> Self {
        Self {
            gpu_name: caps.gpu_name.clone(),
            driver_version: caps.driver_version.clone(),
            driver_branch: caps.driver_branch,
            is_beta: caps.is_beta_driver(),
            is_595_series: caps.is_595_series(),
            descriptor_heap: caps.descriptor_heap,
            descriptor_buffer: caps.descriptor_buffer,
            raw_access_chains: caps.raw_access_chains,
            extended_sparse_address_space: caps.extended_sparse_address_space,
            low_latency2: caps.low_latency2,
            present_timing: caps.present_timing,
        }
    }
}

/// vkd3d-proton installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vkd3dProtonStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub descriptor_heap_support: bool,
}

/// Proton-NV installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtonNvStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
}

/// External tools status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsStatus {
    pub mangohud: bool,
    pub gamemode: bool,
    pub gamemode_running: bool,
}

impl SystemStatus {
    /// Detect full system status
    pub fn detect() -> Self {
        let vulkan = VulkanCapabilities::detect().ok().map(|c| VulkanStatus::from(&c));
        let vkd3d_proton = detect_vkd3d_proton();
        let proton_nv = detect_proton_nv();
        let tools = detect_tools();

        // Determine DX12 readiness
        let (dx12_ready, dx12_ready_reason) = evaluate_dx12_readiness(&vulkan, &vkd3d_proton);

        Self {
            vulkan,
            vkd3d_proton,
            proton_nv,
            tools,
            dx12_ready,
            dx12_ready_reason,
        }
    }

    /// Check if system is ready for VK_EXT_descriptor_heap
    pub fn is_descriptor_heap_ready(&self) -> bool {
        self.dx12_ready
    }
}

/// Detect vkd3d-proton installation
fn detect_vkd3d_proton() -> Option<Vkd3dProtonStatus> {
    // Check common vkd3d-proton locations
    let search_paths = [
        // System installations
        "/usr/share/vkd3d-proton",
        "/usr/local/share/vkd3d-proton",
        // Flatpak
        "/var/lib/flatpak/runtime/org.freedesktop.Platform.VulkanLayer.vkd3d-proton",
        // User installations
        &format!(
            "{}/.local/share/vkd3d-proton",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    // Also check via wine prefix environment
    let wine_vkd3d = std::env::var("VKD3D_PROTON_PATH").ok();

    let mut found_path: Option<PathBuf> = None;
    let mut version: Option<String> = None;

    // Check environment variable first
    if let Some(ref path) = wine_vkd3d {
        let p = PathBuf::from(path);
        if p.exists() {
            found_path = Some(p.clone());
            version = read_vkd3d_version(&p);
        }
    }

    // Check standard paths
    if found_path.is_none() {
        for path_str in &search_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                version = read_vkd3d_version(&path);
                found_path = Some(path);
                break;
            }
        }
    }

    // Try to detect via Proton (vkd3d-proton is bundled)
    if found_path.is_none() {
        if let Some((path, ver)) = detect_vkd3d_from_proton() {
            found_path = Some(path);
            version = Some(ver);
        }
    }

    // Check if version supports descriptor_heap (PR #2805)
    // This requires vkd3d-proton 2.14+ (when merged) or a patched build
    let descriptor_heap_support = version
        .as_ref()
        .is_some_and(|v| version_supports_descriptor_heap(v));

    Some(Vkd3dProtonStatus {
        installed: found_path.is_some(),
        version,
        path: found_path,
        descriptor_heap_support,
    })
}

/// Read vkd3d-proton version from installation
fn read_vkd3d_version(path: &PathBuf) -> Option<String> {
    // Try version file
    let version_file = path.join("version");
    if let Ok(content) = std::fs::read_to_string(&version_file) {
        return Some(content.trim().to_string());
    }

    // Try setup_vkd3d_proton.sh for version info
    let setup_script = path.join("setup_vkd3d_proton.sh");
    if let Ok(content) = std::fs::read_to_string(&setup_script) {
        for line in content.lines() {
            if line.contains("VKD3D_PROTON_VERSION=") {
                if let Some(ver) = line.split('=').nth(1) {
                    return Some(ver.trim_matches('"').to_string());
                }
            }
        }
    }

    None
}

/// Detect vkd3d-proton bundled with Proton
fn detect_vkd3d_from_proton() -> Option<(PathBuf, String)> {
    // Check Steam Proton installations
    let home = std::env::var("HOME").ok()?;
    let steam_path = PathBuf::from(&home).join(".local/share/Steam");

    // Check Proton Experimental
    let proton_exp = steam_path.join("steamapps/common/Proton - Experimental");
    if proton_exp.exists() {
        let vkd3d_path = proton_exp.join("files/lib64/vkd3d-proton");
        if vkd3d_path.exists() {
            // Try to get version from proton_version
            let version_file = proton_exp.join("version");
            if let Ok(content) = std::fs::read_to_string(&version_file) {
                let version = content.lines().next().unwrap_or("unknown").to_string();
                return Some((vkd3d_path, format!("bundled (Proton {})", version)));
            }
            return Some((vkd3d_path, "bundled (Proton Experimental)".to_string()));
        }
    }

    // Check GE-Proton
    let ge_proton_dir = steam_path.join("compatibilitytools.d");
    if ge_proton_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&ge_proton_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("GE-Proton") {
                    let vkd3d_path = entry.path().join("files/lib64/vkd3d-proton");
                    if vkd3d_path.exists() {
                        return Some((vkd3d_path, format!("bundled ({})", name)));
                    }
                }
            }
        }
    }

    None
}

/// Check if vkd3d-proton version supports descriptor_heap
fn version_supports_descriptor_heap(version: &str) -> bool {
    // vkd3d-proton 2.14+ will have descriptor_heap support once PR #2805 merges
    // For now, check for development/git versions or explicit 2.14+

    if version.contains("git") || version.contains("dev") || version.contains("descriptor_heap") {
        return true;
    }

    // Parse version number
    let version_clean = version
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or(version);

    let parts: Vec<u32> = version_clean
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() >= 2 {
        let (major, minor) = (parts[0], parts[1]);
        // 2.14+ expected to have descriptor_heap
        return major > 2 || (major == 2 && minor >= 14);
    }

    false
}

/// Detect Proton-NV installation
fn detect_proton_nv() -> Option<ProtonNvStatus> {
    let mut detector = ProtonNvDetector::new();
    match detector.scan() {
        Ok(_) => {
            if let Some(best) = detector.get_best() {
                Some(ProtonNvStatus {
                    installed: true,
                    version: Some(best.version.clone()),
                    path: Some(best.path.clone()),
                })
            } else {
                Some(ProtonNvStatus {
                    installed: false,
                    version: None,
                    path: None,
                })
            }
        }
        Err(_) => Some(ProtonNvStatus {
            installed: false,
            version: None,
            path: None,
        }),
    }
}

/// Detect external tools
fn detect_tools() -> ToolsStatus {
    let gamemode_running = gamemode::status()
        .map(|s| s.running)
        .unwrap_or(false);

    ToolsStatus {
        mangohud: mangohud::is_installed(),
        gamemode: gamemode::is_installed(),
        gamemode_running,
    }
}

/// Evaluate DX12 readiness based on driver and vkd3d-proton
fn evaluate_dx12_readiness(
    vulkan: &Option<VulkanStatus>,
    vkd3d: &Option<Vkd3dProtonStatus>,
) -> (bool, String) {
    // Check Vulkan driver
    let Some(vk) = vulkan else {
        return (false, "No NVIDIA GPU detected".to_string());
    };

    // Check for descriptor_heap (primary requirement)
    if !vk.descriptor_heap {
        if vk.is_595_series {
            return (
                false,
                format!(
                    "Driver {} is 595 series but VK_EXT_descriptor_heap not available. Reinstall driver?",
                    vk.driver_version
                ),
            );
        } else if vk.is_beta {
            return (
                false,
                format!(
                    "Beta driver {} detected but VK_EXT_descriptor_heap not available. Update to 595.x+",
                    vk.driver_version
                ),
            );
        } else {
            return (
                false,
                format!(
                    "Stable driver {} does not support VK_EXT_descriptor_heap. Update to 595.x beta or wait for stable release",
                    vk.driver_version
                ),
            );
        }
    }

    // Check for extended_sparse_address_space (595 heap fix)
    let has_heap_fix = vk.extended_sparse_address_space;

    // Check vkd3d-proton
    let Some(vkd3d) = vkd3d else {
        return (
            false,
            "VK_EXT_descriptor_heap supported but vkd3d-proton not detected".to_string(),
        );
    };

    if !vkd3d.installed {
        return (
            false,
            "VK_EXT_descriptor_heap supported but vkd3d-proton not installed".to_string(),
        );
    }

    if !vkd3d.descriptor_heap_support {
        // Even without vkd3d support, having the driver ready is partial success
        if has_heap_fix {
            return (
                false,
                format!(
                    "Driver {} ready with heap fix! vkd3d-proton {} needs update (waiting for PR #2805)",
                    vk.driver_version,
                    vkd3d.version.as_deref().unwrap_or("unknown")
                ),
            );
        }
        return (
            false,
            format!(
                "Driver ready but vkd3d-proton {} needs update for descriptor_heap support",
                vkd3d.version.as_deref().unwrap_or("unknown")
            ),
        );
    }

    // Full support!
    if has_heap_fix {
        (
            true,
            format!(
                "Full DX12 optimization: descriptor_heap + heap fix (driver {})",
                vk.driver_version
            ),
        )
    } else {
        (
            true,
            "DX12 descriptor_heap optimization available".to_string(),
        )
    }
}

/// Handle the status command
pub fn handle_status(args: StatusArgs, _manager: &ConfigManager) -> Result<()> {
    let status = SystemStatus::detect();

    if args.check {
        // Exit with code based on descriptor_heap readiness
        if status.is_descriptor_heap_ready() {
            std::process::exit(0);
        } else {
            eprintln!("{}", status.dx12_ready_reason);
            std::process::exit(1);
        }
    }

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&status)?);
        }
        OutputFormat::Text => {
            print_status_text(&status, args.verbose);
        }
    }

    Ok(())
}

/// Print status in human-readable format
fn print_status_text(status: &SystemStatus, verbose: bool) {
    println!("nvproton System Status");
    println!("{}", "=".repeat(50));

    // Vulkan/GPU section
    println!("\nGPU & Driver:");
    if let Some(ref vk) = status.vulkan {
        println!("  GPU: {}", vk.gpu_name);
        print!("  Driver: NVIDIA {}", vk.driver_version);
        if vk.is_beta {
            if vk.is_595_series {
                println!(" (595 beta - DX12 heap fixes)");
            } else {
                println!(" (beta)");
            }
        } else {
            println!();
        }

        if verbose {
            println!("  Driver branch: {}", vk.driver_branch);
        }

        // DX12/vkd3d-proton extensions
        println!("\nDX12 Extensions (vkd3d-proton):");
        print_extension_status("VK_EXT_descriptor_heap", vk.descriptor_heap, true);
        print_extension_status(
            "VK_NV_extended_sparse_address_space",
            vk.extended_sparse_address_space,
            true,
        );
        print_extension_status("VK_EXT_descriptor_buffer", vk.descriptor_buffer, false);
        print_extension_status("VK_NV_raw_access_chains", vk.raw_access_chains, false);

        // Gaming/latency extensions
        println!("\nGaming Extensions:");
        print_extension_status_with_note(
            "VK_NV_low_latency2",
            vk.low_latency2,
            "Reflex 2.0",
        );
        print_extension_status_with_note(
            "VK_EXT_present_timing",
            vk.present_timing,
            "frame pacing",
        );
    } else {
        println!("  No NVIDIA GPU detected");
    }

    // vkd3d-proton section
    println!("\nvkd3d-proton:");
    if let Some(ref vkd3d) = status.vkd3d_proton {
        if vkd3d.installed {
            println!(
                "  Version: {}",
                vkd3d.version.as_deref().unwrap_or("unknown")
            );
            if verbose {
                if let Some(ref path) = vkd3d.path {
                    println!("  Path: {}", path.display());
                }
            }
            print!(
                "  descriptor_heap support: {}",
                if vkd3d.descriptor_heap_support {
                    "yes"
                } else {
                    "no (needs vkd3d-proton 2.14+)"
                }
            );
            println!();
        } else {
            println!("  Not installed");
        }
    } else {
        println!("  Detection failed");
    }

    // Proton-NV section
    println!("\nProton-NV:");
    if let Some(ref pnv) = status.proton_nv {
        if pnv.installed {
            println!(
                "  Version: {}",
                pnv.version.as_deref().unwrap_or("unknown")
            );
            if verbose {
                if let Some(ref path) = pnv.path {
                    println!("  Path: {}", path.display());
                }
            }
        } else {
            println!("  Not installed");
        }
    } else {
        println!("  Detection failed");
    }

    // Tools section
    println!("\nTools:");
    println!(
        "  MangoHud: {}",
        if status.tools.mangohud {
            "installed"
        } else {
            "not found"
        }
    );
    print!(
        "  GameMode: {}",
        if status.tools.gamemode {
            "installed"
        } else {
            "not found"
        }
    );
    if status.tools.gamemode && status.tools.gamemode_running {
        println!(" (daemon running)");
    } else {
        println!();
    }

    // DX12 readiness summary
    println!("\n{}", "=".repeat(50));
    println!("DX12 Optimization Status:");
    if status.dx12_ready {
        println!("  [READY] {}", status.dx12_ready_reason);
    } else {
        println!("  [NOT READY] {}", status.dx12_ready_reason);
    }

    // Recommendations
    if !status.dx12_ready {
        println!("\nRecommendations:");
        print_recommendations(status);
    }
}

fn print_extension_status(name: &str, supported: bool, important: bool) {
    let status = if supported { "supported" } else { "not available" };
    let marker = if important && supported {
        " [DX12 FIX]"
    } else if important && !supported {
        " [WAITING]"
    } else {
        ""
    };
    println!("  {}: {}{}", name, status, marker);
}

fn print_extension_status_with_note(name: &str, supported: bool, note: &str) {
    if supported {
        println!("  {}: supported ({})", name, note);
    } else {
        println!("  {}: not available", name);
    }
}

fn print_recommendations(status: &SystemStatus) {
    if let Some(ref vk) = status.vulkan {
        if !vk.descriptor_heap {
            if vk.is_595_series {
                println!("  - Driver 595 detected but descriptor_heap missing - try reinstalling");
                println!("  - Verify Vulkan ICD is properly configured");
            } else if vk.is_beta {
                println!("  - Update to 595.x beta driver for full DX12 heap fixes");
                println!("  - See: https://developer.nvidia.com/vulkan-driver");
            } else {
                println!("  - Install 595.x beta driver for DX12 optimizations");
                println!("  - Or wait for 600.x stable release");
            }
        } else if !vk.extended_sparse_address_space {
            println!("  - descriptor_heap available but missing heap fix extension");
            println!("  - Update to 595.45+ for VK_NV_extended_sparse_address_space");
        }

        // Reflex 2.0 recommendation
        if !vk.low_latency2 && vk.driver_branch >= 550 {
            println!("  - Update to 595.x for Reflex 2.0 (VK_NV_low_latency2)");
        }
    } else {
        println!("  - Ensure NVIDIA GPU is properly detected");
        println!("  - Check that nvidia-drm kernel module is loaded");
        println!("  - Verify nvidia-utils matches kernel module version");
    }

    if let Some(ref vkd3d) = status.vkd3d_proton {
        if !vkd3d.installed {
            println!("  - Install vkd3d-proton (bundled with Proton/GE-Proton)");
        } else if !vkd3d.descriptor_heap_support {
            println!("  - vkd3d-proton PR #2805 adds descriptor_heap support");
            println!("  - Update vkd3d-proton to 2.14+ when released");
            println!("  - Or build from source: github.com/HansKristian-Work/vkd3d-proton");
        }
    }
}

/// Check for driver updates (for notification system)
#[allow(dead_code)] // Library API for future notification hooks
pub fn check_driver_update() -> Option<String> {
    // Check if a newer driver is available
    if let Ok(caps) = VulkanCapabilities::detect() {
        // Check for 595 with full features
        if caps.driver_branch >= 595 {
            let features = caps.driver_595_features();
            if !features.is_fully_supported() {
                return Some(format!(
                    "Driver {} is 595 series but missing some features. Update to latest 595.x",
                    caps.driver_version
                ));
            }
            // 595 is current best, no update needed
            return None;
        }

        // Recommend 595 for older drivers
        if caps.driver_branch < 595 && !caps.descriptor_heap {
            return Some(format!(
                "Driver {} is outdated. NVIDIA 595.x+ recommended for DX12 optimizations (descriptor_heap + heap fix)",
                caps.driver_version
            ));
        }

        // Has descriptor_heap but not 595 - recommend update for heap fix
        if caps.descriptor_heap && !caps.extended_sparse_address_space {
            return Some(format!(
                "Driver {} has descriptor_heap but missing heap fix. Update to 595.x for VK_NV_extended_sparse_address_space",
                caps.driver_version
            ));
        }
    }

    None
}

/// Get driver readiness level (0-3)
/// 0 = No NVIDIA or very old driver
/// 1 = Has descriptor_heap (580.94+)
/// 2 = Has heap fix (595+)
/// 3 = Full 595 features (descriptor_heap + heap fix + Reflex 2.0)
#[allow(dead_code)] // Library API
pub fn driver_readiness_level() -> u8 {
    if let Ok(caps) = VulkanCapabilities::detect() {
        let features = caps.driver_595_features();

        if features.is_fully_supported() {
            return 3;
        }
        if caps.extended_sparse_address_space {
            return 2;
        }
        if caps.descriptor_heap {
            return 1;
        }
    }
    0
}
