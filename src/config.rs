use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::cli::ConfigCommand;

const CONFIG_FILE_BASENAME: &str = "config.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvConfig {
    #[serde(default)]
    pub library_paths: LibraryPaths,
    #[serde(default)]
    pub detectors: DetectorConfig,
    #[serde(default)]
    pub profile: ProfileConfig,
}

impl Default for NvConfig {
    fn default() -> Self {
        Self {
            library_paths: LibraryPaths::default(),
            detectors: DetectorConfig::default(),
            profile: ProfileConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPaths {
    #[serde(default)]
    pub steam: Option<PathBuf>,
    #[serde(default)]
    pub heroic: Option<PathBuf>,
    #[serde(default)]
    pub lutris: Option<PathBuf>,
}

impl Default for LibraryPaths {
    fn default() -> Self {
        let home = std::env::var("HOME").map(PathBuf::from).ok();
        let steam = home.as_ref().map(|h| h.join(".local/share/Steam"));
        let heroic = home.as_ref().map(|h| h.join(".config/heroic"));
        let lutris = home.as_ref().map(|h| h.join(".local/share/lutris"));
        Self {
            steam,
            heroic,
            lutris,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectorConfig {
    #[serde(default)]
    pub enabled_sources: Vec<String>,
    #[serde(default)]
    pub fingerprint_ignore: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileConfig {
    #[serde(default)]
    pub default_profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub user_config_dir: PathBuf,
    pub games_dir: PathBuf,
    pub profiles_dir: PathBuf,
}

impl ConfigPaths {
    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.user_config_dir).with_context(|| {
            format!(
                "failed to create user config dir at {:?}",
                self.user_config_dir
            )
        })?;
        fs::create_dir_all(&self.games_dir)
            .with_context(|| format!("failed to create games dir at {:?}", self.games_dir))?;
        fs::create_dir_all(&self.profiles_dir)
            .with_context(|| format!("failed to create profiles dir at {:?}", self.profiles_dir))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConfigManager {
    paths: ConfigPaths,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "ghostkellz", "nvproton")
            .context("unable to resolve project directories")?;
        let base_config = project_dirs.config_dir().to_path_buf();
        let paths = ConfigPaths {
            user_config_dir: base_config.clone(),
            games_dir: base_config.join("games"),
            profiles_dir: base_config.join("profiles"),
        };
        Ok(Self { paths })
    }

    pub fn load(&self) -> Result<NvConfig> {
        self.paths.ensure()?;
        let path = self.config_path();
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file at {:?}", path))?;
            let config: NvConfig = if path.extension().and_then(|ext| ext.to_str()) == Some("toml")
            {
                toml::from_str(&contents).context("failed to parse TOML config")?
            } else {
                serde_yaml::from_str(&contents).context("failed to parse YAML config")?
            };
            Ok(config)
        } else {
            let config = NvConfig::default();
            self.save(&config)?;
            Ok(config)
        }
    }

    pub fn save(&self, config: &NvConfig) -> Result<()> {
        self.paths.ensure()?;
        let path = self.config_path();
        let encoded = if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml::to_string_pretty(config).context("failed to serialize config to TOML")?
        } else {
            serde_yaml::to_string(config).context("failed to serialize config to YAML")?
        };
        let mut file = fs::File::create(&path)
            .with_context(|| format!("failed to open config file at {:?}", path))?;
        file.write_all(encoded.as_bytes())
            .with_context(|| format!("failed to write config file at {:?}", path))?;
        Ok(())
    }

    pub fn reset(&self) -> Result<NvConfig> {
        let config = NvConfig::default();
        self.save(&config)?;
        Ok(config)
    }

    pub fn paths(&self) -> &ConfigPaths {
        &self.paths
    }

    pub fn config_path(&self) -> PathBuf {
        self.paths.user_config_dir.join(CONFIG_FILE_BASENAME)
    }
}

pub fn handle_config(
    command: ConfigCommand,
    manager: &ConfigManager,
    config: &mut NvConfig,
) -> Result<()> {
    match command {
        ConfigCommand::Show => {
            println!(
                "{}",
                serde_yaml::to_string(config).context("failed to serialize config for display")?
            );
        }
        ConfigCommand::Paths => {
            println!("config: {:?}", manager.config_path());
            println!("profiles: {:?}", manager.paths().profiles_dir);
            println!("games: {:?}", manager.paths().games_dir);
        }
        ConfigCommand::Reset => {
            *config = manager.reset()?;
            println!("configuration reset to defaults");
        }
    }
    Ok(())
}
