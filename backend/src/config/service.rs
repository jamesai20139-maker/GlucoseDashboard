use crate::{
    config::{model::LocalConfiguration, store::ConfigStore},
    errors::AppError,
};

pub fn configure(
    store: &ConfigStore,
    sheet_id: String,
    sheet_name: String,
    fixture_path: Option<String>,
) -> Result<LocalConfiguration, AppError> {
    let (sheet_id, sheet_gid) = normalize_sheet_reference(&sheet_id)
        .ok_or_else(|| AppError::Invalid("Google Sheet ID 或網址不可為空白。".into()))?;
    if sheet_id.trim().is_empty() {
        return Err(AppError::Invalid("Google Sheet ID 不可為空白。".into()));
    }
    // 保留既有自訂事件與同步時間：configure() 只更新 Sheet 來源，不應清掉其他設定。
    let existing = store.load();
    let config = LocalConfiguration {
        schema_version: 2,
        sheet_id: Some(sheet_id),
        sheet_gid,
        sheet_name: Some(sheet_name),
        fixture_path,
        credential_reference: existing.credential_reference,
        last_successful_sync_at: existing.last_successful_sync_at,
        custom_events: existing.custom_events,
    };
    store.save(&config).map_err(AppError::Internal)?;
    Ok(config)
}

pub fn normalize_sheet_reference(input: &str) -> Option<(String, Option<String>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("https://docs.google.com/spreadsheets/d/")
        || trimmed.starts_with("http://docs.google.com/spreadsheets/d/")
    {
        let without_prefix = trimmed.split("/d/").nth(1)?;
        let sheet_id = without_prefix.split('/').next()?;
        let sheet_gid = trimmed
            .split("gid=")
            .nth(1)
            .and_then(|value| value.split(['&', '#']).next())
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty());
        return Some((sheet_id.to_string(), sheet_gid));
    }
    Some((trimmed.to_string(), None))
}
