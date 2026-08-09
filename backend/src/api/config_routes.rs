use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::{model::CustomEventConfig, service},
    diagnostics::checks,
    errors::AppError,
    ingestion::sync_service::SyncService,
};

#[derive(Serialize)]
pub struct ConfigStatus {
    pub configured: bool,
    pub credential_store: &'static str,
    pub schema_version: u32,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<String>,
    pub last_successful_sync_at: Option<String>,
    pub custom_events: Vec<CustomEventConfig>,
}

fn config_status(config: &crate::config::model::LocalConfiguration) -> ConfigStatus {
    let store = crate::auth::credential_store::CredentialStore;
    ConfigStatus {
        configured: config.is_configured(),
        credential_store: store.status(),
        schema_version: config.schema_version,
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: config.fixture_path.clone(),
        last_successful_sync_at: config.last_successful_sync_at.clone(),
        custom_events: config.custom_events.clone(),
    }
}

pub async fn status(State(state): State<ApiState>) -> Json<ConfigStatus> {
    let config = state.config.load();
    Json(config_status(&config))
}

#[derive(Deserialize)]
pub struct ConfigureRequest {
    pub sheet_id: String,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<String>,
}

pub async fn configure(
    State(state): State<ApiState>,
    Json(request): Json<ConfigureRequest>,
) -> Result<Json<crate::config::model::LocalConfiguration>, AppError> {
    let sheet_name = request.sheet_name.unwrap_or_else(|| "Sheet1".into());
    let saved = service::configure(
        &state.config,
        request.sheet_id,
        sheet_name,
        request.fixture_path,
    )?;
    // 設定變更後舊快取簽章不再相符，主動清空避免殘留。
    *state.records_cache.write().await = None;
    Ok(Json(saved))
}

/// 新增或更新自訂事件關鍵字。同 label 視為更新（覆蓋閾值）。驗證：label 非空且不
/// 與 6 個內建事件衝突、閾值在 20–600 且 low < high。失敗回 400；成功後清空快取、
/// 回傳更新後的設定狀態。
#[derive(Deserialize)]
pub struct AddCustomEventRequest {
    pub label: String,
    pub low_threshold: i32,
    pub high_threshold: i32,
}

/// 內建事件名稱（自訂關鍵字不得與之衝突）。
pub const BUILTIN_EVENT_LABELS: [&str; 6] =
    ["空腹血糖", "午餐前", "午餐後", "晚餐前", "晚餐後", "睡前"];

/// 驗證自訂事件關鍵字。回傳正規化後的 `(label, low, high)` 或 `AppError::Invalid`。
/// 規則：label 非空且不與內建衝突、閾值在 20–600、low < high。
fn validate_custom_event(label: &str, low: i32, high: i32) -> Result<(String, i32, i32), AppError> {
    let label = label.trim().to_string();
    if label.is_empty() || BUILTIN_EVENT_LABELS.contains(&label.as_str()) {
        return Err(AppError::Invalid(
            "事件關鍵字不可為空，且不可與內建事件同名。".into(),
        ));
    }
    if !(20..=600).contains(&low) || !(20..=600).contains(&high) || low >= high {
        return Err(AppError::Invalid(
            "閾值須介於 20–600，且正常下限須小於上限。".into(),
        ));
    }
    Ok((label, low, high))
}

pub async fn add_custom_event(
    State(state): State<ApiState>,
    Json(request): Json<AddCustomEventRequest>,
) -> Result<Json<ConfigStatus>, AppError> {
    let (label, low, high) = validate_custom_event(
        &request.label,
        request.low_threshold,
        request.high_threshold,
    )?;

    let mut config = state.config.load();
    let new_event = CustomEventConfig {
        label: label.clone(),
        low_threshold: low,
        high_threshold: high,
    };
    // upsert：既有同 label 則覆蓋閾值，否則新增。
    if let Some(existing) = config.custom_events.iter_mut().find(|c| c.label == label) {
        *existing = new_event;
    } else {
        config.custom_events.push(new_event);
    }
    config.schema_version = 2;
    state.config.save(&config).map_err(AppError::Internal)?;
    // 自訂事件變更影響解析結果，主動清空快取。
    *state.records_cache.write().await = None;
    Ok(Json(config_status(&config)))
}

/// 刪除指定 label 的自訂事件關鍵字。不存在視為成功（冪等）。清空快取後回傳設定狀態。
pub async fn delete_custom_event(
    State(state): State<ApiState>,
    Path(label): Path<String>,
) -> Result<Json<ConfigStatus>, AppError> {
    let mut config = state.config.load();
    let before = config.custom_events.len();
    config.custom_events.retain(|c| c.label != label);
    if config.custom_events.len() != before {
        state.config.save(&config).map_err(AppError::Internal)?;
        *state.records_cache.write().await = None;
    }
    Ok(Json(config_status(&config)))
}

pub async fn diagnostics(State(state): State<ApiState>) -> Json<Vec<checks::CheckResult>> {
    let cache = state.records_cache.read().await;
    let cache_info = cache.as_ref().map(|c| (c.fetched_at, c.records.len()));
    Json(checks::run(&state.config, cache_info))
}

#[derive(Serialize)]
pub struct ConnectionTestResponse {
    pub status: &'static str,
    pub ok: bool,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub url: Option<String>,
    pub http_status: Option<u16>,
    pub record_count: Option<usize>,
    pub issue_count: Option<usize>,
    pub message: String,
    pub detail: Option<String>,
}

pub async fn test_connection(
    State(state): State<ApiState>,
) -> Result<Json<ConnectionTestResponse>, AppError> {
    let config = state.config.load();
    let service = SyncService {
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: None,
        custom_events: config.custom_events.clone(),
    };
    let report = service.test_google_sheet_connection().await?;
    Ok(Json(ConnectionTestResponse {
        status: "succeeded",
        ok: report.ok,
        sheet_id: report.sheet_id,
        sheet_gid: report.sheet_gid,
        sheet_name: report.sheet_name,
        url: report.url,
        http_status: report.http_status,
        record_count: report.record_count,
        issue_count: report.issue_count,
        message: report.message,
        detail: report.detail,
    }))
}

#[cfg(test)]
mod tests {
    use super::validate_custom_event;
    use crate::errors::AppError;

    #[test]
    fn accepts_valid_custom_event() {
        let (label, low, high) = validate_custom_event("運動後", 70, 139).unwrap();
        assert_eq!(label, "運動後");
        assert_eq!((low, high), (70, 139));
        // 前後空白會被 trim。
        let (label, _, _) = validate_custom_event("  運動後  ", 70, 139).unwrap();
        assert_eq!(label, "運動後");
    }

    #[test]
    fn rejects_empty_label() {
        assert!(validate_custom_event("", 70, 139).is_err());
        assert!(validate_custom_event("   ", 70, 139).is_err());
    }

    #[test]
    fn rejects_builtin_label_conflict() {
        for builtin in super::BUILTIN_EVENT_LABELS {
            assert!(validate_custom_event(builtin, 70, 139).is_err());
        }
    }

    #[test]
    fn rejects_thresholds_out_of_range() {
        // low 下限 20。
        assert!(validate_custom_event("運動後", 19, 139).is_err());
        // high 上限 600。
        assert!(validate_custom_event("運動後", 70, 601).is_err());
    }

    #[test]
    fn rejects_low_not_less_than_high() {
        assert!(validate_custom_event("運動後", 140, 140).is_err());
        assert!(validate_custom_event("運動後", 141, 140).is_err());
    }

    /// 透過臨時設定檔驗證 add/delete 的 upsert 與持久化行為。
    fn temp_state() -> super::ApiState {
        use std::sync::Arc;
        use tokio::sync::RwLock;
        let path = std::env::temp_dir().join(format!(
            "glucose-dashboard-custom-event-test-{}.json",
            std::process::id()
        ));
        std::env::set_var("GLUCOSE_CONFIG_PATH", &path);
        let _ = std::fs::remove_file(&path);
        super::ApiState {
            config: crate::config::store::ConfigStore::default(),
            records_cache: Arc::new(RwLock::new(None)),
        }
    }

    #[tokio::test]
    async fn add_then_upsert_and_delete_persist() {
        let state = temp_state();
        // 新增「運動後」70/139。
        let added = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 70,
                high_threshold: 139,
            }),
        )
        .await
        .unwrap();
        assert_eq!(added.custom_events.len(), 1);
        assert_eq!(added.custom_events[0].label, "運動後");
        assert_eq!(added.custom_events[0].low_threshold, 70);
        assert_eq!(added.custom_events[0].high_threshold, 139);

        // 同 label 新增 → upsert 覆蓋閾值，數量不增加。
        let updated = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 80,
                high_threshold: 120,
            }),
        )
        .await
        .unwrap();
        assert_eq!(updated.custom_events.len(), 1);
        assert_eq!(updated.custom_events[0].low_threshold, 80);
        assert_eq!(updated.custom_events[0].high_threshold, 120);

        // 設定檔確實持久化：重新載入仍應有該事件。
        let reloaded = state.config.load();
        assert_eq!(reloaded.custom_events.len(), 1);

        // 刪除後歸零，且刪除不存在的 label 為冪等（不報錯）。
        let deleted = super::delete_custom_event(
            axum::extract::State(state.clone()),
            axum::extract::Path("運動後".to_string()),
        )
        .await
        .unwrap();
        assert!(deleted.custom_events.is_empty());
        let _ = super::delete_custom_event(
            axum::extract::State(state),
            axum::extract::Path("不存在".to_string()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn add_invalid_event_returns_400() {
        let state = temp_state();
        let result = super::add_custom_event(
            axum::extract::State(state),
            axum::Json(super::AddCustomEventRequest {
                label: "空腹血糖".into(),
                low_threshold: 70,
                high_threshold: 139,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }
}
