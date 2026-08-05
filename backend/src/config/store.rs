use crate::config::model::LocalConfiguration;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl Default for ConfigStore {
    fn default() -> Self {
        let path = std::env::var_os("GLUCOSE_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".glucose-dashboard.json"));
        Self { path }
    }
}

impl ConfigStore {
    pub fn load(&self) -> LocalConfiguration {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, config: &LocalConfiguration) -> Result<(), String> {
        let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
        fs::write(&self.path, text).map_err(|error| error.to_string())
    }
}
