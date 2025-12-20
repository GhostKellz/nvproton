use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

use super::model::{ProfileDocument, ResolvedProfile};

pub struct ProfileManager {
    root: PathBuf,
}

impl ProfileManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();
        if !self.root.exists() {
            return Ok(profiles);
        }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml")
                && let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str())
            {
                profiles.push(stem.to_string());
            }
        }
        profiles.sort();
        Ok(profiles)
    }

    pub fn load(&self, name: &str) -> Result<ProfileDocument> {
        let path = self.path_for(name);
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read profile at {:?}", path))?;
        let document: ProfileDocument =
            serde_yaml::from_str(&contents).context("failed to parse profile document")?;
        Ok(document)
    }

    pub fn save(&self, document: &ProfileDocument) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create profiles directory at {:?}", self.root))?;
        let path = self.path_for(&document.name);
        let encoded =
            serde_yaml::to_string(document).context("failed to encode profile document")?;
        let mut file = fs::File::create(&path)
            .with_context(|| format!("failed to open profile file at {:?}", path))?;
        file.write_all(encoded.as_bytes())
            .with_context(|| format!("failed to write profile file at {:?}", path))?;
        Ok(())
    }

    pub fn exists(&self, name: &str) -> bool {
        self.path_for(name).exists()
    }

    pub fn resolve(&self, name: &str) -> Result<ResolvedProfile> {
        let mut chain = Vec::new();
        let mut cursor = Some(name.to_string());
        while let Some(current_name) = cursor {
            if chain.iter().any(|(existing, _)| existing == &current_name) {
                anyhow::bail!("profile inheritance loop detected at '{}'", current_name);
            }
            let document = self.load(&current_name)?;
            cursor = document.extends.clone();
            chain.push((current_name, document));
        }
        let mut merged = Mapping::new();
        for (_, document) in chain.iter().rev() {
            merge_mapping(&mut merged, &document.settings);
        }
        Ok(ResolvedProfile {
            name: name.to_string(),
            settings: Value::Mapping(merged),
        })
    }

    fn path_for(&self, name: &str) -> PathBuf {
        self.root.join(format!("{}.yaml", name))
    }
}

fn merge_mapping(target: &mut Mapping, source: &Mapping) {
    for (key, value) in source {
        match value {
            Value::Mapping(child) => {
                let entry = target
                    .entry(key.clone())
                    .or_insert_with(|| Value::Mapping(Mapping::new()));
                if let Value::Mapping(existing) = entry {
                    merge_mapping(existing, child);
                } else {
                    *entry = Value::Mapping(child.clone());
                }
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}
