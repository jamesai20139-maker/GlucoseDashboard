use crate::{
    config::{
        model::{
            LocalConfiguration, CURRENT_SCHEMA_VERSION, DEFAULT_EVENT_KEYWORDS_SHEET_NAME,
            DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME,
        },
        store::ConfigStore,
    },
    errors::AppError,
};

/// 套用 Google Sheet 設定。`sheet_id` 可為完整網址或 ID；`sheet_name` 為資料
/// 工作表名稱；`fixture_path` 為本機 CSV 回退路徑。`event_keywords_sheet_name`
/// 與 `glucose_standards_sheet_name` 為兩個設定工作表名稱，空白或 None 則套用
/// 預設常數。保留既有 `credential_reference` 與 `last_successful_sync_at`。
/// 自 schema 4 起 `configure()` 不再處理 `custom_events`/`event_thresholds`
/// （該兩項改由 Sheet 即時衍生，不持久化）。
pub fn configure(
    store: &ConfigStore,
    sheet_id: String,
    sheet_name: String,
    fixture_path: Option<String>,
    event_keywords_sheet_name: Option<String>,
    glucose_standards_sheet_name: Option<String>,
) -> Result<LocalConfiguration, AppError> {
    let (sheet_id, sheet_gid) = normalize_sheet_reference(&sheet_id)
        .ok_or_else(|| AppError::Invalid("Google Sheet ID 或網址不可為空白。".into()))?;
    if sheet_id.trim().is_empty() {
        return Err(AppError::Invalid("Google Sheet ID 不可為空白。".into()));
    }
    let existing = store.load();
    // 兩個設定工作表名稱：trim 後若空白則套預設常數。
    let event_keywords_sheet_name = event_keywords_sheet_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_EVENT_KEYWORDS_SHEET_NAME.to_string());
    let glucose_standards_sheet_name = glucose_standards_sheet_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME.to_string());
    let mut config = LocalConfiguration {
        schema_version: CURRENT_SCHEMA_VERSION,
        sheet_id: Some(sheet_id),
        sheet_gid,
        sheet_name: Some(sheet_name),
        fixture_path,
        credential_reference: existing.credential_reference,
        last_successful_sync_at: existing.last_successful_sync_at,
        event_keywords_sheet_name: Some(event_keywords_sheet_name),
        glucose_standards_sheet_name: Some(glucose_standards_sheet_name),
    };
    config.normalize();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> ConfigStore {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "glucose-configure-test-{}-{}.json",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        ConfigStore::from_path(path)
    }

    #[test]
    fn configure_defaults_worksheet_names_when_blank() {
        let store = temp_store();
        let config = configure(
            &store,
            "ABC".into(),
            "Sheet1".into(),
            None,
            Some("   ".into()),
            None,
        )
        .unwrap();
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some(DEFAULT_EVENT_KEYWORDS_SHEET_NAME)
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME)
        );
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn configure_preserves_explicit_worksheet_names() {
        let store = temp_store();
        let config = configure(
            &store,
            "ABC".into(),
            "Sheet1".into(),
            None,
            Some("我的關鍵字".into()),
            Some("我的標準值".into()),
        )
        .unwrap();
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some("我的關鍵字")
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some("我的標準值")
        );
    }

    #[test]
    fn configure_rejects_empty_sheet_id() {
        let store = temp_store();
        let result = configure(&store, "   ".into(), "Sheet1".into(), None, None, None);
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[test]
    fn normalize_sheet_reference_extracts_id_and_gid() {
        let (id, gid) = normalize_sheet_reference(
            "https://docs.google.com/spreadsheets/d/1JzMjBq/edit#gid=1855776901",
        )
        .unwrap();
        assert_eq!(id, "1JzMjBq");
        assert_eq!(gid.as_deref(), Some("1855776901"));
    }
}
