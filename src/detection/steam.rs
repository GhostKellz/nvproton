use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use glob::glob;
use regex::Regex;
use walkdir::WalkDir;

use super::fingerprint;
use super::{DetectedGame, DetectionContext, GameSource};

pub struct SteamDetector;

impl SteamDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(
        &self,
        ctx: &DetectionContext<'_>,
        include_fingerprint: bool,
    ) -> Result<Vec<DetectedGame>> {
        let mut games = Vec::new();
        let steam_path = match ctx.config.library_paths.steam.as_ref() {
            Some(path) => path.clone(),
            None => return Ok(games),
        };
        if !steam_path.exists() {
            return Ok(games);
        }
        let library_dirs = read_library_folders(&steam_path)?;
        for library in library_dirs {
            let manifest_pattern = library.join("steamapps").join("appmanifest_*.acf");
            for entry in glob(manifest_pattern.to_string_lossy().as_ref())? {
                let path = entry?;
                if let Some(manifest) = parse_manifest(&path)? {
                    // Skip Steam internals (Proton, Runtime, Redistributables)
                    if is_excluded_appid(&manifest.appid) {
                        continue;
                    }

                    let install_dir = library
                        .join("steamapps")
                        .join("common")
                        .join(&manifest.installdir);
                    let executable = locate_primary_executable(&install_dir);
                    let fingerprint_value = if include_fingerprint {
                        executable
                            .as_ref()
                            .and_then(|exe| fingerprint::fingerprint_file(exe).ok())
                    } else {
                        None
                    };
                    let mut metadata = manifest.metadata.clone();
                    if let Some(appid) = manifest.metadata.get("appid").cloned() {
                        metadata.insert("appid".into(), appid);
                    }
                    games.push(DetectedGame {
                        source: GameSource::Steam,
                        id: manifest.appid,
                        name: manifest.name,
                        install_dir,
                        executable,
                        fingerprint: fingerprint_value,
                        metadata,
                    });
                }
            }
        }
        Ok(games)
    }
}

struct Manifest {
    appid: String,
    name: String,
    installdir: String,
    metadata: std::collections::HashMap<String, String>,
}

fn read_library_folders(steam_root: &Path) -> Result<Vec<PathBuf>> {
    let library_file = steam_root.join("steamapps").join("libraryfolders.vdf");
    if !library_file.exists() {
        return Ok(vec![steam_root.to_path_buf()]);
    }
    let content = fs::read_to_string(&library_file)
        .with_context(|| format!("failed to read {:?}", library_file))?;
    let regex = Regex::new(r#"path"\s+"([^"]+)"#)?;
    let mut directories = vec![steam_root.to_path_buf()];
    for captures in regex.captures_iter(&content) {
        let path = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
        let expanded = PathBuf::from(path.replace("\\", "/"));
        if expanded.exists() {
            directories.push(expanded);
        }
    }
    directories.sort();
    directories.dedup();
    Ok(directories)
}

fn parse_manifest(path: &Path) -> Result<Option<Manifest>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read steam manifest at {:?}", path))?;
    let mut appid = None;
    let mut name = None;
    let mut installdir = None;
    let mut metadata = std::collections::HashMap::new();
    let key_regex = Regex::new(r#"(?P<key>[^"]+)"\s+"(?P<value>[^"]*)"#)?;
    for captures in key_regex.captures_iter(&content) {
        let key = captures.name("key").unwrap().as_str();
        let value = captures.name("value").unwrap().as_str().to_string();
        match key {
            "appid" => appid = Some(value.clone()),
            "name" => name = Some(value.clone()),
            "installdir" => installdir = Some(value.clone()),
            _ => {}
        }
        metadata.insert(key.to_string(), value);
    }
    Ok(match (appid, name, installdir) {
        (Some(appid), Some(name), Some(installdir)) => Some(Manifest {
            appid,
            name,
            installdir,
            metadata,
        }),
        _ => None,
    })
}

/// AppIDs that are Steam internals, not actual games
const EXCLUDED_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1493710", // Proton Experimental
    "1628350", // Steam Linux Runtime 3.0 (sniper)
    "1887720", // Proton 8.0
    "2180100", // Proton 9.0
    "2348590", // Proton 9.0 (another)
    "3658110", // Proton 10.0
    "1391110", // Steam Linux Runtime
    "2805730", // Steam Linux Runtime (soldier)
];

pub fn is_excluded_appid(appid: &str) -> bool {
    EXCLUDED_APPIDS.contains(&appid)
}

fn locate_primary_executable(install_dir: &Path) -> Option<PathBuf> {
    if !install_dir.exists() {
        return None;
    }

    // Collect all .exe files first
    let mut exe_candidates: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(install_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        // Only consider actual Windows executables
        if ext == "exe" {
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Skip known non-game executables
            if is_launcher_or_tool(&filename) {
                continue;
            }

            exe_candidates.push(path.to_path_buf());
        }
    }

    // Prioritize executables by likelihood of being the main game
    exe_candidates.sort_by(|a, b| {
        let a_score = score_executable(a, install_dir);
        let b_score = score_executable(b, install_dir);
        b_score.cmp(&a_score) // Higher score first
    });

    exe_candidates.into_iter().next()
}

/// Check if executable is a launcher/tool rather than the main game
fn is_launcher_or_tool(filename: &str) -> bool {
    const SKIP_PATTERNS: &[&str] = &[
        "unins",
        "uninst",
        "setup",
        "install",
        "update",
        "patch",
        "crash",
        "reporter",
        "helper",
        "service",
        "launcher",
        "easyanticheat",
        "battleye",
        "dxsetup",
        "vcredist",
        "dotnet",
        "directx",
        "physx",
        "ue4prereq",
        "redist",
        "cef",
        "subprocess",
        "browser",
        "webhelper",
        "upc",
        "uplay",
        "origin",
        "epic",
    ];

    for pattern in SKIP_PATTERNS {
        if filename.contains(pattern) {
            return true;
        }
    }
    false
}

/// Score an executable by how likely it is to be the main game
fn score_executable(path: &Path, install_dir: &Path) -> i32 {
    let mut score = 0;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let dirname = install_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Bonus: exe name matches directory name (common pattern)
    let exe_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if dirname.contains(&exe_stem) || exe_stem.contains(&dirname.replace(" ", "")) {
        score += 50;
    }

    // Bonus: in root or bin directory (not deep subdirectories)
    let depth = path
        .strip_prefix(install_dir)
        .map(|p| p.components().count())
        .unwrap_or(10);
    if depth <= 1 {
        score += 30;
    } else if depth == 2 {
        // Common pattern: game/bin/game.exe
        if let Some(parent) = path.parent() {
            let parent_name = parent.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if parent_name == "bin" || parent_name == "Binaries" || parent_name == "x64" {
                score += 25;
            }
        }
    }

    // Bonus: common game executable patterns
    if filename.ends_with("-win64-shipping.exe") || filename.ends_with("_win64.exe") {
        score += 20;
    }
    if filename.contains("game") || filename.contains("client") {
        score += 10;
    }

    // Penalty: likely not the main game
    if filename.contains("server") && !filename.contains("dedicated") {
        score -= 10;
    }

    // Penalty: very small files are usually not the game
    if let Ok(metadata) = path.metadata() {
        if metadata.len() < 1_000_000 {
            // Less than 1MB
            score -= 20;
        } else if metadata.len() > 50_000_000 {
            // Over 50MB, likely the real game
            score += 15;
        }
    }

    score
}
