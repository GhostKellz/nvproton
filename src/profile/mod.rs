mod manager;
mod model;

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::fs;

use crate::cli::{
    OutputFormat, ProfileArgs, ProfileCommand, ProfileCreateArgs, ProfileExportArgs,
    ProfileImportArgs, ProfileNameArgs, ProfileSetArgs,
};
use crate::config::{ConfigManager, NvConfig};

pub use manager::ProfileManager;
pub use model::ProfileDocument;

pub fn handle_profile(
    args: ProfileArgs,
    manager: &ConfigManager,
    _config: &mut NvConfig,
) -> Result<()> {
    let profile_manager = ProfileManager::new(manager.paths().profiles_dir.clone());
    match args.command {
        ProfileCommand::List => {
            for name in profile_manager.list()? {
                println!("{}", name);
            }
        }
        ProfileCommand::Show(ProfileNameArgs { name }) => {
            let resolved = profile_manager.resolve(&name)?;
            println!("{}", serde_yaml::to_string(&resolved.settings)?)
        }
        ProfileCommand::Create(ProfileCreateArgs { name, base, values }) => {
            if profile_manager.exists(&name) {
                anyhow::bail!("profile '{}' already exists", name);
            }
            let mut document = ProfileDocument::new(name.clone());
            document.extends = base;
            apply_sets(&mut document, &values)?;
            profile_manager.save(&document)?;
            println!("profile '{}' created", name);
        }
        ProfileCommand::Set(ProfileSetArgs { name, values }) => {
            let mut document = profile_manager.load(&name)?;
            apply_sets(&mut document, &values)?;
            profile_manager.save(&document)?;
            println!("profile '{}' updated", name);
        }
        ProfileCommand::Import(ProfileImportArgs { path, name }) => {
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("failed to read profile from {:?}", path))?;
            let mut document: ProfileDocument = serde_yaml::from_str(&contents)
                .or_else(|_| serde_json::from_str(&contents))
                .context("failed to parse profile document")?;
            if let Some(name) = name {
                document.name = name;
            }
            profile_manager.save(&document)?;
            println!("profile '{}' imported", document.name);
        }
        ProfileCommand::Export(ProfileExportArgs { name, format, path }) => {
            let document = profile_manager.load(&name)?;
            let encoded = match format {
                OutputFormat::Text | OutputFormat::Yaml => serde_yaml::to_string(&document)?,
                OutputFormat::Json => serde_json::to_string_pretty(&document)?,
            };
            if let Some(path) = path {
                fs::write(&path, encoded)
                    .with_context(|| format!("failed to write profile export to {:?}", path))?;
                println!("profile '{}' exported to {:?}", name, path);
            } else {
                println!("{}", encoded);
            }
        }
    }
    Ok(())
}

fn apply_sets(document: &mut ProfileDocument, values: &[(String, String)]) -> Result<()> {
    for (key, value) in values {
        set_nested_value(&mut document.settings, key, Value::String(value.clone()))?;
    }
    Ok(())
}

fn set_nested_value(root: &mut Mapping, key: &str, value: Value) -> Result<()> {
    let mut parts = key.split('.').peekable();
    let mut current = root;
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current.insert(Value::String(part.to_string()), value.clone());
            return Ok(());
        }
        current = current
            .entry(Value::String(part.to_string()))
            .or_insert_with(|| Value::Mapping(Mapping::new()))
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("key '{}' conflicts with non-mapping value", key))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_nested_values() {
        let mut document = ProfileDocument::new("test".into());
        apply_sets(
            &mut document,
            &[
                ("graphics.fsr".into(), "balanced".into()),
                ("audio.volume".into(), "90".into()),
            ],
        )
        .expect("apply sets");
        let resolved = document
            .settings
            .get(&Value::String("graphics".into()))
            .unwrap();
        let graphics = resolved.as_mapping().unwrap();
        assert_eq!(
            graphics[&Value::String("fsr".into())],
            Value::String("balanced".into())
        );
    }
}
