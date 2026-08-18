use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;

use super::router::ApiState;
use crate::{
    config::model::EventThreshold,
    domain::CustomEvent,
    errors::AppError,
    ingestion::{
        settings_loader::{builtin_fallback_settings, load_sheet_settings_with_fetcher},
        sync_service::SyncService,
    },
};
// EventThreshold 位於 config::model（非 domain）。

#[derive(Serialize)]
pub struct SyncResponse {
    pub status: &'static str,
    pub records: Vec<crate::domain::GlucoseRecord>,
    pub issues: Vec<crate::domain::DataQualityIssue>,
    pub last_successful_sync_at: Option<String>,
    /// 即時衍生的自訂事件關鍵字。
    pub custom_events: Vec<CustomEvent>,
    /// 即時衍生的血糖標準值。
    pub event_thresholds: Vec<EventThreshold>,
}

pub async fn sync(State(state): State<ApiState>) -> Result<Json<SyncResponse>, AppError> {
    let config = state.config.load();
    let signature = super::router::source_signature(&config);

    // 每次同步都重新衍生設定（已連結 Sheet 抓兩工作表；本機 CSV 退回內建預設）。
    let settings = if config
        .sheet_id
        .as_ref()
        .map(|id| !id.trim().is_empty())
        .unwrap_or(false)
    {
        let sheet_id = config.sheet_id.clone().unwrap();
        let keywords_name = config
            .event_keywords_sheet_name
            .clone()
            .unwrap_or_else(|| "事件關鍵字設定".into());
        let standards_name = config
            .glucose_standards_sheet_name
            .clone()
            .unwrap_or_else(|| "血糖標準值設定".into());
        load_sheet_settings_with_fetcher(
            state.sheet_fetcher.as_ref(),
            &sheet_id,
            &keywords_name,
            &standards_name,
        )
        .await?
    } else {
        builtin_fallback_settings()
    };

    let service = SyncService {
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: config.fixture_path.clone().map(Into::into),
        custom_events: settings.custom_events.clone(),
    };
    let (records, issues, table_rows) = service
        .load_with_fetcher(state.sheet_fetcher.as_ref())
        .await?;
    // 強制重抓後寫入快取，讓後續切換區間不再重抓 Sheet。
    *state.records_cache.write().await = Some(super::router::RecordsCache {
        records: records.clone(),
        table_rows,
        issues: issues.clone(),
        custom_events: settings.custom_events.clone(),
        event_thresholds: settings.event_thresholds.clone(),
        fetched_at: Utc::now(),
        source_signature: signature,
    });
    // 設定與資料皆成功後才記錄同步時間。
    let timestamp = Utc::now().to_rfc3339();
    let mut updated = config;
    updated.last_successful_sync_at = Some(timestamp.clone());
    let _ = state.config.save(&updated);
    Ok(Json(SyncResponse {
        status: "succeeded",
        records,
        issues,
        last_successful_sync_at: Some(timestamp),
        custom_events: settings.custom_events,
        event_thresholds: settings.event_thresholds,
    }))
}
