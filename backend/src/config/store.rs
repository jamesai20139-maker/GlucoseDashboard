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
    /// 以指定路徑建構（主要供測試注入，避免並行測試經環境變數競爭同一設定檔）。
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> LocalConfiguration {
        let mut config: LocalConfiguration = fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        // 舊設定檔 migration：補齊內建閾值、去重、排序、升 schema 版本。
        config.normalize();
        config
    }

    pub fn save(&self, config: &LocalConfiguration) -> Result<(), String> {
        let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
        fs::write(&self.path, text).map_err(|error| error.to_string())
    }
}
