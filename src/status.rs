//! System status and driver readiness reporting
//!
//! Provides comprehensive system status including:
//! - Vulkan driver and extension support (DX12 heap-fix features, 595+)
//! - vkd3d-proton installation and version
//! - Proton-NV detection
//! - DX12 readiness (descriptor_heap + extended sparse support)
//! - Reflex 2.0 and frame pacing capabilities

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cli::{OutputFormat, StatusArgs};
use crate::config::ConfigManager;
use crate::detection::VulkanCapabilities;
use crate::detection::proton_nv::ProtonNvDetector;
use crate::gamemode;
use crate::mangohud;

/// Comprehensive system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub vulkan: Option<VulkanStatus>,
    pub vkd3d_proton: Option<Vkd3dProtonStatus>,
    pub proton_nv: Option<ProtonNvStatus>,
    pub tools: ToolsStatus,
    pub audio: crate::audio::AudioStatus,
    pub encoder: crate::streaming::EncoderStatus,
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
    /// Driver branch is new enough (595+) to ship the DX12 heap-fix feature set
    pub dx12_heap_branch: bool,
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
            dx12_heap_branch: caps.has_dx12_heap_branch(),
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
        let vulkan = VulkanCapabilities::detect()
            .ok()
            .map(|c| VulkanStatus::from(&c));
        let vkd3d_proton = detect_vkd3d_proton();
        let proton_nv = detect_proton_nv();
        let tools = detect_tools();
        let audio = crate::audio::status();
        let encoder = crate::streaming::status();

        // Determine DX12 readiness
        let (dx12_ready, dx12_ready_reason) = evaluate_dx12_readiness(&vulkan, &vkd3d_proton);

        Self {
            vulkan,
            vkd3d_proton,
            proton_nv,
            tools,
            audio,
            encoder,
            dx12_ready,
            dx12_ready_reason,
        }
    }

    /// Check if system is ready for VK_EXT_descriptor_heap
    pub fn is_descriptor_heap_ready(&self) -> bool {
        self.dx12_ready
    }
}

/// Detect vkd3d-proton installation.
///
/// vkd3d-proton is almost always bundled inside a Proton build rather than
/// installed standalone, so this checks (in order): an explicit
/// `VKD3D_PROTON_PATH`, standalone system/user installs, then every Proton
/// build under each Steam root.
fn detect_vkd3d_proton() -> Option<Vkd3dProtonStatus> {
    // 1. Explicit override via environment.
    if let Some(path) = std::env::var("VKD3D_PROTON_PATH").ok().map(PathBuf::from)
        && path.exists()
    {
        let version = read_vkd3d_version(&path);
        let descriptor_heap_support = version
            .as_deref()
            .is_some_and(version_supports_descriptor_heap);
        return Some(Vkd3dProtonStatus {
            installed: true,
            version,
            path: Some(path),
            descriptor_heap_support,
        });
    }

    // 2. Standalone system/user installs.
    let home = std::env::var("HOME").unwrap_or_default();
    let standalone_paths = [
        "/usr/share/vkd3d-proton".to_string(),
        "/usr/local/share/vkd3d-proton".to_string(),
        "/var/lib/flatpak/runtime/org.freedesktop.Platform.VulkanLayer.vkd3d-proton".to_string(),
        format!("{home}/.local/share/vkd3d-proton"),
    ];
    for path_str in &standalone_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let version = read_vkd3d_version(&path);
            let descriptor_heap_support = version
                .as_deref()
                .is_some_and(version_supports_descriptor_heap);
            return Some(Vkd3dProtonStatus {
                installed: true,
                version,
                path: Some(path),
                descriptor_heap_support,
            });
        }
    }

    // 3. Bundled inside a Proton build (the common case).
    if let Some((path, version)) = detect_vkd3d_from_proton() {
        // The split `vkd3d-proton/` runtime layout corresponds to modern
        // vkd3d-proton builds that ship VK_EXT_descriptor_heap support.
        let descriptor_heap_support = path.to_string_lossy().contains("vkd3d-proton");
        return Some(Vkd3dProtonStatus {
            installed: true,
            version: Some(version),
            path: Some(path),
            descriptor_heap_support,
        });
    }

    Some(Vkd3dProtonStatus {
        installed: false,
        version: None,
        path: None,
        descriptor_heap_support: false,
    })
}

/// Read vkd3d-proton version from installation
fn read_vkd3d_version(path: &Path) -> Option<String> {
    // Try version file
    let version_file = path.join("version");
    if let Ok(content) = std::fs::read_to_string(&version_file) {
        return Some(content.trim().to_string());
    }

    // Try setup_vkd3d_proton.sh for version info
    let setup_script = path.join("setup_vkd3d_proton.sh");
    if let Ok(content) = std::fs::read_to_string(&setup_script) {
        for line in content.lines() {
            if line.contains("VKD3D_PROTON_VERSION=")
                && let Some(ver) = line.split('=').nth(1)
            {
                return Some(ver.trim_matches('"').to_string());
            }
        }
    }

    None
}

/// Candidate Steam data roots, de-duplicated by canonical path.
fn steam_roots() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{home}/.local/share/Steam"),
        format!("{home}/.steam/steam"),
        format!("{home}/.steam/root"),
        format!("{home}/.var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];

    let mut roots = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for candidate in candidates {
        let path = PathBuf::from(&candidate);
        if !path.exists() {
            continue;
        }
        let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if seen.insert(canonical.clone()) {
            roots.push(canonical);
        }
    }
    roots
}

/// Relative paths (within a Proton build's directory) where the vkd3d-proton
/// d3d12 runtime DLL may live, newest layout first.
const VKD3D_DLL_RELPATHS: &[&str] = &[
    "files/lib/wine/vkd3d-proton/x86_64-windows/d3d12.dll",
    "files/lib64/wine/vkd3d-proton/x86_64-windows/d3d12.dll",
    "files/lib/wine/x86_64-windows/d3d12.dll",
    "files/lib64/wine/x86_64-windows/d3d12.dll",
];

/// Find the vkd3d-proton runtime DLL inside a single Proton build directory.
fn vkd3d_dll_in_proton(proton_dir: &Path) -> Option<PathBuf> {
    VKD3D_DLL_RELPATHS
        .iter()
        .map(|rel| proton_dir.join(rel))
        .find(|p| p.exists())
}

/// Read the human-readable Proton version label from a build directory.
/// Proton `version` files look like: `1780064122 experimental-11.0-20260529`.
fn read_proton_version(proton_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(proton_dir.join("version")).ok()?;
    let line = content.lines().next()?;
    Some(line.split_whitespace().last().unwrap_or(line).to_string())
}

/// Detect vkd3d-proton bundled inside a Proton build.
///
/// Scans both Valve Proton installs (`steamapps/common/*roton*`) and custom
/// builds in `compatibilitytools.d` (GE-Proton, proton-tkg, proton-cachyos, ...)
/// across every Steam root.
fn detect_vkd3d_from_proton() -> Option<(PathBuf, String)> {
    for steam in steam_roots() {
        // Valve Proton builds under steamapps/common.
        if let Ok(entries) = std::fs::read_dir(steam.join("steamapps/common")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.to_lowercase().contains("proton") {
                    continue;
                }
                if let Some(dll) = vkd3d_dll_in_proton(&entry.path()) {
                    let version = read_proton_version(&entry.path()).unwrap_or(name);
                    return Some((dll, format!("bundled (Proton {})", version)));
                }
            }
        }

        // Custom Proton/Wine builds under compatibilitytools.d.
        if let Ok(entries) = std::fs::read_dir(steam.join("compatibilitytools.d")) {
            for entry in entries.flatten() {
                if let Some(dll) = vkd3d_dll_in_proton(&entry.path()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let version = read_proton_version(&entry.path())
                        .map(|v| format!("bundled ({name} {v})"))
                        .unwrap_or_else(|| format!("bundled ({name})"));
                    return Some((dll, version));
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
    let gamemode_running = gamemode::status().map(|s| s.running).unwrap_or(false);

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
        let hint = if vk.dx12_heap_branch {
            "driver branch should expose it - try reinstalling or check the Vulkan ICD"
        } else {
            "requires NVIDIA driver branch 595 or newer"
        };
        return (
            false,
            format!(
                "Driver {} does not expose VK_EXT_descriptor_heap ({})",
                vk.driver_version, hint
            ),
        );
    }

    // Check for extended_sparse_address_space (DX12 heap fix)
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
            println!("{}", serde_norway::to_string(&status)?);
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
            if vk.dx12_heap_branch {
                println!(" (beta - DX12 heap fixes)");
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
        print_extension_status_with_note("VK_NV_low_latency2", vk.low_latency2, "Reflex 2.0");
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
            if verbose && let Some(ref path) = vkd3d.path {
                println!("  Path: {}", path.display());
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
            println!("  Version: {}", pnv.version.as_deref().unwrap_or("unknown"));
            if verbose && let Some(ref path) = pnv.path {
                println!("  Path: {}", path.display());
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

    // Noise suppression (ghostwave) section
    println!("\nNoise Suppression (ghostwave):");
    if status.audio.available {
        match status.audio.processing_mode {
            Some(ref mode) => println!("  Available: yes ({})", mode),
            None => println!("  Available: yes"),
        }
        if status.audio.rtx_acceleration {
            println!("  RTX acceleration: yes");
        }
    } else {
        println!("  Not built (rebuild with --features noise-suppression)");
    }

    // Capture / encode (ghoststream) section
    println!("\nCapture/Encode (ghoststream):");
    if status.encoder.available {
        println!(
            "  NVENC: {}",
            if status.encoder.nvenc {
                "available"
            } else {
                "not available"
            }
        );
        if let Some(ref gpu) = status.encoder.gpu_name {
            println!("  Encoder GPU: {}", gpu);
        }
        if !status.encoder.codecs.is_empty() {
            println!("  NVENC codecs: {}", status.encoder.codecs.join(", "));
        }
    } else {
        println!("  Not built (rebuild with --features streaming)");
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
    let status = if supported {
        "supported"
    } else {
        "not available"
    };
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
            if vk.dx12_heap_branch {
                println!(
                    "  - Driver branch {} should expose VK_EXT_descriptor_heap - try reinstalling",
                    vk.driver_branch
                );
                println!("  - Verify Vulkan ICD is properly configured");
            } else {
                println!(
                    "  - Update to NVIDIA driver branch 595+ for DX12 descriptor_heap optimizations"
                );
                println!("  - See: https://developer.nvidia.com/vulkan-driver");
            }
        } else if !vk.extended_sparse_address_space {
            println!("  - descriptor_heap available but missing the heap-fix extension");
            println!(
                "  - Update to the latest driver in your branch for VK_NV_extended_sparse_address_space"
            );
        }

        // Reflex 2.0 recommendation
        if !vk.low_latency2 && vk.driver_branch >= 550 {
            println!("  - Update to driver branch 595+ for Reflex 2.0 (VK_NV_low_latency2)");
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
        // DX12-capable branch (595+): only flag if some features are missing.
        if caps.has_dx12_heap_branch() {
            let features = caps.dx12_features();
            if !features.is_fully_supported() {
                return Some(format!(
                    "Driver {} is on a DX12-capable branch but missing some features. Update to the latest driver in your branch",
                    caps.driver_version
                ));
            }
            return None;
        }

        // Older branch without descriptor_heap.
        if !caps.descriptor_heap {
            return Some(format!(
                "Driver {} is outdated. NVIDIA branch 595+ recommended for DX12 optimizations (descriptor_heap + heap fix)",
                caps.driver_version
            ));
        }

        // Has descriptor_heap but missing the heap fix.
        if caps.descriptor_heap && !caps.extended_sparse_address_space {
            return Some(format!(
                "Driver {} has descriptor_heap but missing the heap fix. Update to branch 595+ for VK_NV_extended_sparse_address_space",
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
/// 3 = Full DX12 heap-fix set (descriptor_heap + heap fix + Reflex 2.0)
#[allow(dead_code)] // Library API
pub fn driver_readiness_level() -> u8 {
    if let Ok(caps) = VulkanCapabilities::detect() {
        let features = caps.dx12_features();

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
