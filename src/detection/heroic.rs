use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use glob::glob;
use serde::Deserialize;

use super::fingerprint;
use super::{DetectedGame, DetectionContext, GameSource};

pub struct HeroicDetector;

impl HeroicDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(
        &self,
        ctx: &DetectionContext<'_>,
        include_fingerprint: bool,
    ) -> Result<Vec<DetectedGame>> {
        let heroic_root = match ctx.config.library_paths.heroic.as_ref() {
            Some(path) => path.clone(),
            None => return Ok(Vec::new()),
        };
        if !heroic_root.exists() {
            return Ok(Vec::new());
        }
        let mut games = Vec::new();
        let pattern = heroic_root.join("store").join("*").join("library.json");
        for entry in glob(pattern.to_string_lossy().as_ref())? {
            let path = entry?;
            games.extend(parse_library_file(&path, include_fingerprint)?);
        }
        Ok(games)
    }
}

fn parse_library_file(path: &Path, include_fingerprint: bool) -> Result<Vec<DetectedGame>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read heroic library at {:?}", path))?;
    let games: HeroicLibrary = serde_json::from_str(&contents)
        .or_else(|_| {
            serde_json::from_str::<HeroicLegacyLibrary>(&contents).map(|legacy| legacy.into())
        })
        .context("failed to parse heroic library json")?;
    let mut detected = Vec::new();
    for entry in games.games {
        if entry.install_path.is_none() {
            continue;
        }
        let install_dir = PathBuf::from(entry.install_path.unwrap());
        let identifier = if !entry.identifier.is_empty() {
            entry.identifier.clone()
        } else if let Some(app_name) = entry.app_name.clone() {
            app_name
        } else if !entry.title.is_empty() {
            entry.title.clone()
        } else {
            continue;
        };
        let display_name = if entry.title.is_empty() {
            identifier.clone()
        } else {
            entry.title.clone()
        };
        let executable = entry
            .executable
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| locate_executable_hint(&install_dir, entry.launch_options.as_ref()))
            .filter(|p| p.exists());
        let fingerprint_value = if include_fingerprint {
            executable
                .as_ref()
                .and_then(|exe| fingerprint::fingerprint_file(exe).ok())
        } else {
            None
        };
        let mut metadata = HashMap::new();
        if let Some(app_name) = entry.app_name.clone() {
            metadata.insert("app_name".into(), app_name);
        }
        if let Some(platform) = entry.platform.clone() {
            metadata.insert("platform".into(), platform);
        }
        detected.push(DetectedGame {
            source: GameSource::Heroic,
            id: identifier,
            name: display_name,
            install_dir,
            executable,
            fingerprint: fingerprint_value,
            metadata,
        });
    }
    Ok(detected)
}

fn locate_executable_hint(install_dir: &Path, hint: Option<&String>) -> Option<PathBuf> {
    match hint {
        Some(hint) if !hint.is_empty() => {
            let mut candidate = install_dir.to_path_buf();
            candidate.push(hint);
            Some(candidate)
        }
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct HeroicLibrary {
    games: Vec<HeroicGame>,
}

#[derive(Debug, Deserialize)]
struct HeroicLegacyLibrary {
    #[serde(default)]
    library: Vec<HeroicGame>,
}

impl From<HeroicLegacyLibrary> for HeroicLibrary {
    fn from(value: HeroicLegacyLibrary) -> Self {
        Self {
            games: value.library,
        }
    }
}

#[derive(Debug, Deserialize)]
struct HeroicGame {
    #[serde(default)]
    identifier: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    app_name: Option<String>,
    #[serde(default, alias = "install_dir")]
    install_path: Option<String>,
    #[serde(default)]
    executable: Option<String>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    launch_options: Option<String>,
}
