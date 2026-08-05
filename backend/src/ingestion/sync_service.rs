use std::path::PathBuf;

use super::sheet_parser::parse_csv;
use crate::{
    domain::{DataQualityIssue, GlucoseRecord},
    errors::AppError,
};

#[derive(Clone, Default)]
pub struct SyncService {
    pub fixture_path: Option<PathBuf>,
}

impl SyncService {
    pub fn load(&self) -> Result<(Vec<GlucoseRecord>, Vec<DataQualityIssue>), AppError> {
        let path = self
            .fixture_path
            .clone()
            .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
        let text = std::fs::read_to_string(path)
            .map_err(|_| AppError::Sync("無法讀取資料來源。".into()))?;
        Ok(parse_csv(&text))
    }
}
