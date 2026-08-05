use std::path::PathBuf;

use super::sheet_parser::parse_csv;
use crate::{
    domain::{DataQualityIssue, GlucoseRecord},
    errors::AppError,
};

#[derive(Clone, Default)]
pub struct SyncService {
    pub sheet_id: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<PathBuf>,
}

impl SyncService {
    pub fn load(&self) -> Result<(Vec<GlucoseRecord>, Vec<DataQualityIssue>), AppError> {
        let text = if let Some(path) = self.fixture_path.clone() {
            std::fs::read_to_string(path).map_err(|_| AppError::Sync("無法讀取資料來源。".into()))?
        } else {
            let sheet_id = self
                .sheet_id
                .clone()
                .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
            let sheet_name = self.sheet_name.clone().unwrap_or_else(|| "Sheet1".into());
            fetch_google_sheet_csv(&sheet_id, &sheet_name)?
        };
        Ok(parse_csv(&text))
    }
}

fn fetch_google_sheet_csv(sheet_id: &str, sheet_name: &str) -> Result<String, AppError> {
    let encoded_sheet_name = urlencoding::encode(sheet_name);
    let url = format!(
        "https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet={encoded_sheet_name}"
    );
    let response = reqwest::blocking::get(url).map_err(|_| {
        AppError::Sync("無法連線到 Google Sheet，請確認網路、分享權限或工作表是否公開可讀。".into())
    })?;
    if !response.status().is_success() {
        return Err(AppError::Sync(format!(
            "Google Sheet 讀取失敗，HTTP 狀態碼 {}。",
            response.status()
        )));
    }
    response
        .text()
        .map_err(|_| AppError::Sync("無法讀取 Google Sheet 內容。".into()))
}
