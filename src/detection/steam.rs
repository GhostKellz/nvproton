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

fn locate_primary_executable(install_dir: &Path) -> Option<PathBuf> {
    if !install_dir.exists() {
        return None;
    }
    for entry in WalkDir::new(install_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let lowercase = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(lowercase.as_str(), "exe" | "sh" | "appimage") {
            return Some(path.to_path_buf());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if entry
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                return Some(path.to_path_buf());
            }
        }
    }
    None
}
