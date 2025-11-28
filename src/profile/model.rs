use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDocument {
    pub name: String,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub settings: Mapping,
}

impl ProfileDocument {
    pub fn new(name: String) -> Self {
        Self {
            name,
            extends: None,
            settings: Mapping::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedProfile {
    pub name: String,
    pub settings: Value,
}
