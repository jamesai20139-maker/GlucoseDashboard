use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::{
        model::{CustomEventConfig, EventThreshold},
        service,
    },
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
    /// 每個事件的前端顯示標準範圍（6 內建 + 自訂事件，固定順序）。
    pub event_thresholds: Vec<EventThreshold>,
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
        event_thresholds: config.event_thresholds.clone(),
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

/// 新增自訂事件關鍵字。此分頁只負責關鍵字新增/刪除；閾值改由「血糖標準值」分頁
/// （`POST /api/event-thresholds`）統一設定。驗證：label 非空且不與 6 個內建事件
/// 衝突。成功後清空快取、回傳更新後的設定狀態。新事件以預設顯示標準 70–139 補入
/// `event_thresholds`（已存在則不重設）。
#[derive(Deserialize)]
pub struct AddCustomEventRequest {
    pub label: String,
    /// 舊欄位，保留以維持 wire 相容；此端點不再使用閾值（改由 event-thresholds 設定）。
    #[serde(default)]
    pub low_threshold: i32,
    #[serde(default)]
    pub high_threshold: i32,
}

/// 內建事件名稱（自訂關鍵字不得與之衝突）。
pub const BUILTIN_EVENT_LABELS: [&str; 6] =
    ["空腹血糖", "午餐前", "午餐後", "晚餐前", "晚餐後", "睡前"];

/// 驗證自訂事件關鍵字 label。回傳 trim 後的 label 或 `AppError::Invalid`。
/// 規則：label 非空且不與內建衝突。
fn validate_custom_event_label(label: &str) -> Result<String, AppError> {
    let label = label.trim().to_string();
    if label.is_empty() || BUILTIN_EVENT_LABELS.contains(&label.as_str()) {
        return Err(AppError::Invalid(
            "事件關鍵字不可為空，且不可與內建事件同名。".into(),
        ));
    }
    Ok(label)
}

pub async fn add_custom_event(
    State(state): State<ApiState>,
    Json(request): Json<AddCustomEventRequest>,
) -> Result<Json<ConfigStatus>, AppError> {
    let label = validate_custom_event_label(&request.label)?;

    let mut config = state.config.load();
    // 新事件加入 custom_events（後端固定 fallback 70/139，不影響摘要統計）。
    if !config.custom_events.iter().any(|c| c.label == label) {
        config.custom_events.push(CustomEventConfig {
            label: label.clone(),
            low_threshold: crate::config::model::CUSTOM_EVENT_FALLBACK_LOW,
            high_threshold: crate::config::model::CUSTOM_EVENT_FALLBACK_HIGH,
        });
        // 同步在 event_thresholds 補預設顯示標準（已存在則不重設）。
        if !config.event_thresholds.iter().any(|t| t.label == label) {
            config.event_thresholds.push(EventThreshold {
                label: label.clone(),
                low: crate::config::model::CUSTOM_EVENT_FALLBACK_LOW,
                high: crate::config::model::CUSTOM_EVENT_FALLBACK_HIGH,
            });
        }
    }
    // 同 label 再次新增視為冪等：不重設既有閾值。
    config.schema_version = crate::config::model::CURRENT_SCHEMA_VERSION;
    state.config.save(&config).map_err(AppError::Internal)?;
    // 自訂事件變更影響解析結果，主動清空快取。
    *state.records_cache.write().await = None;
    Ok(Json(config_status(&config)))
}

/// 刪除指定 label 的自訂事件關鍵字。不存在視為成功（冪等）。同時從
/// `custom_events` 與 `event_thresholds` 移除。拒絕刪除內建事件。清空快取後回傳
/// 設定狀態。
pub async fn delete_custom_event(
    State(state): State<ApiState>,
    Path(label): Path<String>,
) -> Result<Json<ConfigStatus>, AppError> {
    // 內建事件不可刪除。
    if BUILTIN_EVENT_LABELS.contains(&label.as_str()) {
        return Err(AppError::Invalid("內建事件不可刪除。".into()));
    }
    let mut config = state.config.load();
    let before = config.custom_events.len();
    config.custom_events.retain(|c| c.label != label);
    config.event_thresholds.retain(|t| t.label != label);
    if config.custom_events.len() != before {
        config.schema_version = crate::config::model::CURRENT_SCHEMA_VERSION;
        state.config.save(&config).map_err(AppError::Internal)?;
        *state.records_cache.write().await = None;
    }
    Ok(Json(config_status(&config)))
}

/// 批次覆寫全部事件的前端顯示標準範圍。驗證：每筆 label 非空且為內建或現存自訂
/// 事件、不可重複、low/high 在 20–600、low < high；提交集合須完整含 6 內建 +
/// 全部現存自訂事件。只更新 `event_thresholds`、不動 `custom_events`；不影響後端
/// 摘要統計。成功後清空快取、回傳設定狀態。
#[derive(Deserialize)]
pub struct UpdateEventThresholdsRequest {
    pub event_thresholds: Vec<EventThreshold>,
}

pub async fn update_event_thresholds(
    State(state): State<ApiState>,
    Json(request): Json<UpdateEventThresholdsRequest>,
) -> Result<Json<ConfigStatus>, AppError> {
    let mut config = state.config.load();
    let allowed: Vec<String> = BUILTIN_EVENT_LABELS
        .iter()
        .map(|s| s.to_string())
        .chain(config.custom_events.iter().map(|c| c.label.clone()))
        .collect();

    // 驗證每筆。
    let mut seen: Vec<String> = Vec::new();
    for t in &request.event_thresholds {
        let label = t.label.trim().to_string();
        if label.is_empty() {
            return Err(AppError::Invalid("事件標準值的事件名稱不可為空。".into()));
        }
        if !allowed.iter().any(|a| a == &label) {
            return Err(AppError::Invalid(format!(
                "事件「{label}」非內建或現存自訂事件，無法設定標準值。"
            )));
        }
        if seen.iter().any(|s| s == &label) {
            return Err(AppError::Invalid(format!("事件「{label}」重複出現。")));
        }
        seen.push(label);
        if !(20..=600).contains(&t.low) || !(20..=600).contains(&t.high) {
            return Err(AppError::Invalid("閾值須介於 20–600。".into()));
        }
        if t.low >= t.high {
            return Err(AppError::Invalid("正常下限須小於上限。".into()));
        }
    }

    // 提交集合須完整：6 內建 + 全部現存自訂事件。
    for required in &allowed {
        if !request
            .event_thresholds
            .iter()
            .any(|t| t.label.trim() == required.trim())
        {
            return Err(AppError::Invalid(format!(
                "缺少事件「{required}」的標準值，需包含全部內建與自訂事件。"
            )));
        }
    }

    // 套用並正規化（去重、排序）。
    config.event_thresholds = request
        .event_thresholds
        .iter()
        .map(|t| EventThreshold {
            label: t.label.trim().to_string(),
            low: t.low,
            high: t.high,
        })
        .collect();
    config.normalize();
    config.schema_version = crate::config::model::CURRENT_SCHEMA_VERSION;
    state.config.save(&config).map_err(AppError::Internal)?;
    // 閾值只影響前端顯示顏色，但依決策仍清空快取。
    *state.records_cache.write().await = None;
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
    use super::{validate_custom_event_label, EventThreshold, UpdateEventThresholdsRequest};
    use crate::errors::AppError;

    #[test]
    fn accepts_valid_label() {
        assert_eq!(validate_custom_event_label("運動後").unwrap(), "運動後");
        // 前後空白會被 trim。
        assert_eq!(validate_custom_event_label("  運動後  ").unwrap(), "運動後");
    }

    #[test]
    fn rejects_empty_label() {
        assert!(validate_custom_event_label("").is_err());
        assert!(validate_custom_event_label("   ").is_err());
    }

    #[test]
    fn rejects_builtin_label_conflict() {
        for builtin in super::BUILTIN_EVENT_LABELS {
            assert!(validate_custom_event_label(builtin).is_err());
        }
    }

    /// 透過臨時設定檔驗證 add/delete 的持久化行為。每次呼叫產生唯一檔名並直接
    /// 注入 `ConfigStore`，避免並行測試經環境變數競爭同一設定檔造成狀態污染。
    fn temp_state() -> super::ApiState {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "glucose-dashboard-custom-event-test-{}-{}.json",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        super::ApiState {
            config: crate::config::store::ConfigStore::from_path(path),
            records_cache: Arc::new(RwLock::new(None)),
        }
    }

    fn threshold(label: &str, low: i32, high: i32) -> EventThreshold {
        EventThreshold {
            label: label.into(),
            low,
            high,
        }
    }

    #[tokio::test]
    async fn add_custom_event_creates_default_threshold() {
        let state = temp_state();
        let added = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        // custom_events 新增一筆，後端固定 fallback 70/139。
        assert_eq!(added.custom_events.len(), 1);
        assert_eq!(added.custom_events[0].label, "運動後");
        assert_eq!(added.custom_events[0].low_threshold, 70);
        assert_eq!(added.custom_events[0].high_threshold, 139);
        // event_thresholds 含 6 內建 + 運動後預設 70/139。
        let sport = added
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (70, 139));
        assert_eq!(added.event_thresholds.len(), 7);

        // 設定檔持久化：重新載入仍應有該事件與閾值。
        let reloaded = state.config.load();
        assert_eq!(reloaded.custom_events.len(), 1);
        assert!(reloaded
            .event_thresholds
            .iter()
            .any(|t| t.label == "運動後"));
    }

    #[tokio::test]
    async fn add_custom_event_is_idempotent_and_keeps_thresholds() {
        let state = temp_state();
        // 先新增運動後。
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        // 把運動後閾值改成 80/120。
        let all = super::update_event_thresholds(
            axum::extract::State(state.clone()),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: six_builtins_with("運動後", 80, 120),
            }),
        )
        .await
        .unwrap();
        let sport = all
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (80, 120));

        // 再次 add 運動後：冪等，不重設閾值。
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        let reloaded = state.config.load();
        let sport = reloaded
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (80, 120));
    }

    #[tokio::test]
    async fn delete_custom_event_removes_threshold() {
        let state = temp_state();
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        let deleted = super::delete_custom_event(
            axum::extract::State(state.clone()),
            axum::extract::Path("運動後".to_string()),
        )
        .await
        .unwrap();
        assert!(deleted.custom_events.is_empty());
        assert!(!deleted.event_thresholds.iter().any(|t| t.label == "運動後"));
        // 刪除不存在的 label 為冪等（不報錯）。
        let _ = super::delete_custom_event(
            axum::extract::State(state),
            axum::extract::Path("不存在".to_string()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn delete_builtin_event_returns_400() {
        let state = temp_state();
        let result = super::delete_custom_event(
            axum::extract::State(state),
            axum::extract::Path("空腹血糖".to_string()),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn add_invalid_event_returns_400() {
        let state = temp_state();
        let result = super::add_custom_event(
            axum::extract::State(state),
            axum::Json(super::AddCustomEventRequest {
                label: "空腹血糖".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    /// 6 內建預設閾值 + 一個自訂事件（用於 update 測試）。
    fn six_builtins_with(label: &str, low: i32, high: i32) -> Vec<EventThreshold> {
        vec![
            threshold("空腹血糖", 70, 100),
            threshold("午餐前", 70, 101),
            threshold("午餐後", 70, 140),
            threshold("晚餐前", 70, 101),
            threshold("晚餐後", 70, 140),
            threshold("睡前", 70, 140),
            threshold(label, low, high),
        ]
    }

    #[tokio::test]
    async fn update_thresholds_accepts_all_events() {
        let state = temp_state();
        // 先新增一個自訂事件，使 allowed 含它。
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        let result = super::update_event_thresholds(
            axum::extract::State(state.clone()),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: six_builtins_with("運動後", 80, 120),
            }),
        )
        .await
        .unwrap();
        let sport = result
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (80, 120));
        // custom_events 不應被改動。
        assert_eq!(result.custom_events.len(), 1);
    }

    #[tokio::test]
    async fn update_thresholds_does_not_modify_custom_events() {
        let state = temp_state();
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        let before: Vec<String> = state
            .config
            .load()
            .custom_events
            .iter()
            .map(|c| c.label.clone())
            .collect();
        let _ = super::update_event_thresholds(
            axum::extract::State(state.clone()),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: six_builtins_with("運動後", 80, 120),
            }),
        )
        .await
        .unwrap();
        let after: Vec<String> = state
            .config
            .load()
            .custom_events
            .iter()
            .map(|c| c.label.clone())
            .collect();
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn update_thresholds_rejects_unknown_event() {
        let state = temp_state();
        let result = super::update_event_thresholds(
            axum::extract::State(state),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: six_builtins_with("不存在的事件", 70, 139),
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn update_thresholds_rejects_missing_event() {
        let state = temp_state();
        let _ = super::add_custom_event(
            axum::extract::State(state.clone()),
            axum::Json(super::AddCustomEventRequest {
                label: "運動後".into(),
                low_threshold: 0,
                high_threshold: 0,
            }),
        )
        .await
        .unwrap();
        // 只送 6 內建，缺運動後。
        let result = super::update_event_thresholds(
            axum::extract::State(state),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: vec![
                    threshold("空腹血糖", 70, 100),
                    threshold("午餐前", 70, 101),
                    threshold("午餐後", 70, 140),
                    threshold("晚餐前", 70, 101),
                    threshold("晚餐後", 70, 140),
                    threshold("睡前", 70, 140),
                ],
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn update_thresholds_rejects_duplicate() {
        let state = temp_state();
        let mut list = six_builtins_with("運動後", 80, 120);
        list.push(threshold("空腹血糖", 70, 99));
        let result = super::update_event_thresholds(
            axum::extract::State(state),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: list,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn update_thresholds_rejects_out_of_range() {
        let state = temp_state();
        // low=19 低於 20。
        let list = six_builtins_with("運動後", 19, 139);
        let result = super::update_event_thresholds(
            axum::extract::State(state.clone()),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: list,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
        // high=601 超過 600。
        let list = six_builtins_with("運動後", 70, 601);
        let result = super::update_event_thresholds(
            axum::extract::State(state),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: list,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn update_thresholds_rejects_low_not_less_than_high() {
        let state = temp_state();
        let list = six_builtins_with("運動後", 140, 140);
        let result = super::update_event_thresholds(
            axum::extract::State(state),
            axum::Json(UpdateEventThresholdsRequest {
                event_thresholds: list,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }
}
