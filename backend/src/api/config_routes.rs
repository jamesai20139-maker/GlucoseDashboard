use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::{
        model::{CustomEventConfig, EventThreshold},
        service,
    },
    diagnostics::checks,
    errors::AppError,
    ingestion::sync_service::{GoogleSheetFetcher, ReqwestGoogleSheetFetcher, SyncService},
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
    pub event_keywords_sheet_name: Option<String>,
    pub glucose_standards_sheet_name: Option<String>,
    /// 即時衍生的自訂事件關鍵字（自 schema 4 起不持久化，取自快取暫存值；
    /// 首次同步前為空）。權威來源為 DashboardResponse/SyncResponse。
    pub custom_events: Vec<CustomEventConfig>,
    /// 即時衍生的血糖標準值（同上，取自快取；首次同步前為空）。
    pub event_thresholds: Vec<EventThreshold>,
}

/// 組裝 `ConfigStatus`：持久化的來源欄位 + 快取中暫存的衍生設定。
/// 自 schema 4 起 `custom_events`/`event_thresholds` 不再持久化，改由 Sheet
/// 即時衍生並暫存於 `records_cache`；首次同步前快取為空故回傳空陣列。
async fn config_status(
    state: &ApiState,
    config: &crate::config::model::LocalConfiguration,
) -> ConfigStatus {
    let store = crate::auth::credential_store::CredentialStore;
    let (custom_events, event_thresholds) = state
        .records_cache
        .read()
        .await
        .as_ref()
        .map(|c| (c.custom_events.clone(), c.event_thresholds.clone()))
        .unwrap_or_default();
    ConfigStatus {
        configured: config.is_configured(),
        credential_store: store.status(),
        schema_version: config.schema_version,
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: config.fixture_path.clone(),
        last_successful_sync_at: config.last_successful_sync_at.clone(),
        event_keywords_sheet_name: config.event_keywords_sheet_name.clone(),
        glucose_standards_sheet_name: config.glucose_standards_sheet_name.clone(),
        custom_events,
        event_thresholds,
    }
}

pub async fn status(State(state): State<ApiState>) -> Json<ConfigStatus> {
    let config = state.config.load();
    Json(config_status(&state, &config).await)
}

#[derive(Deserialize)]
pub struct ConfigureRequest {
    pub sheet_id: String,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<String>,
    pub event_keywords_sheet_name: Option<String>,
    pub glucose_standards_sheet_name: Option<String>,
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
        request.event_keywords_sheet_name,
        request.glucose_standards_sheet_name,
    )?;
    // 設定變更後舊快取簽章不再相符，主動清空避免殘留。
    *state.records_cache.write().await = None;
    Ok(Json(saved))
}

pub async fn diagnostics(State(state): State<ApiState>) -> Json<Vec<checks::CheckResult>> {
    let cache = state.records_cache.read().await;
    let cache_info = cache.as_ref().map(|c| (c.fetched_at, c.records.len()));
    Json(checks::run(&state.config, cache_info))
}

/// 單一工作表的連線測試報告。缺失/格式錯誤回 `ok:false`，不拋 AppError，
/// 讓使用者看到哪個分頁失敗。
#[derive(Serialize)]
pub struct WorksheetConnectionReport {
    pub ok: bool,
    pub sheet_name: Option<String>,
    pub url: Option<String>,
    pub http_status: Option<u16>,
    pub row_count: Option<usize>,
    pub header_valid: bool,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectionTestResponse {
    pub status: &'static str,
    pub ok: bool,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    /// 資料工作表報告。
    pub data_sheet: WorksheetConnectionReport,
    /// 「事件關鍵字設定」工作表報告。
    pub event_keywords_sheet: WorksheetConnectionReport,
    /// 「血糖標準值設定」工作表報告。
    pub glucose_standards_sheet: WorksheetConnectionReport,
}

/// 測試連線：分別檢查資料工作表與兩個設定工作表，回傳個別報告。
/// 三表皆 ok 時頂層 ok=true。任一失敗回 ok=false 但仍 200（非阻塞診斷）。
pub async fn test_connection(
    State(state): State<ApiState>,
) -> Result<Json<ConnectionTestResponse>, AppError> {
    let config = state.config.load();
    let raw_sheet_id = config
        .sheet_id
        .clone()
        .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
    let (sheet_id, parsed_gid) =
        crate::config::service::normalize_sheet_reference(&raw_sheet_id)
            .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
    let sheet_gid = config.sheet_gid.clone().or(parsed_gid);
    let sheet_name = config.sheet_name.clone().unwrap_or_else(|| "Sheet1".into());
    let keywords_name = config
        .event_keywords_sheet_name
        .clone()
        .unwrap_or_else(|| "事件關鍵字設定".into());
    let standards_name = config
        .glucose_standards_sheet_name
        .clone()
        .unwrap_or_else(|| "血糖標準值設定".into());

    let fetcher = state.sheet_fetcher.as_ref();

    // 資料工作表：沿用 SyncService 連線測試邏輯。
    let data_service = SyncService {
        sheet_id: Some(sheet_id.clone()),
        sheet_gid: sheet_gid.clone(),
        sheet_name: Some(sheet_name.clone()),
        fixture_path: None,
        custom_events: Vec::new(),
    };
    let data_report = test_data_sheet(&data_service, fetcher).await;

    // 兩個設定工作表：個別抓取並驗證 header/可解析性。
    let keywords_report = test_settings_worksheet(
        fetcher,
        &sheet_id,
        &keywords_name,
        "事件關鍵字設定",
        SettingsKind::Keywords,
    )
    .await;
    let standards_report = test_settings_worksheet(
        fetcher,
        &sheet_id,
        &standards_name,
        "血糖標準值設定",
        SettingsKind::Standards,
    )
    .await;

    let ok = data_report.ok && keywords_report.ok && standards_report.ok;
    Ok(Json(ConnectionTestResponse {
        status: "succeeded",
        ok,
        sheet_id: Some(sheet_id),
        sheet_gid,
        data_sheet: data_report,
        event_keywords_sheet: keywords_report,
        glucose_standards_sheet: standards_report,
    }))
}

enum SettingsKind {
    Keywords,
    Standards,
}

/// 測試資料工作表連線與可解析性（沿用既有 ConnectionReport 邏輯）。
async fn test_data_sheet(
    service: &SyncService,
    _fetcher: &dyn GoogleSheetFetcher,
) -> WorksheetConnectionReport {
    // 資料表連線測試用生產 fetcher 即可（test_connection 僅於已設定真實 Sheet 時呼叫）。
    let prod = ReqwestGoogleSheetFetcher;
    let raw_sheet_id = match service.sheet_id.clone() {
        Some(id) => id,
        None => {
            return WorksheetConnectionReport {
                ok: false,
                sheet_name: service.sheet_name.clone(),
                url: None,
                http_status: None,
                row_count: None,
                header_valid: false,
                message: "尚未設定 Google Sheet。".into(),
                detail: None,
            }
        }
    };
    let (sheet_id, parsed_gid) =
        match crate::config::service::normalize_sheet_reference(&raw_sheet_id) {
            Some(pair) => pair,
            None => {
                return WorksheetConnectionReport {
                    ok: false,
                    sheet_name: service.sheet_name.clone(),
                    url: None,
                    http_status: None,
                    row_count: None,
                    header_valid: false,
                    message: "Google Sheet ID 或網址無效。".into(),
                    detail: None,
                }
            }
        };
    let sheet_gid = service.sheet_gid.clone().or(parsed_gid);
    let sheet_name = service
        .sheet_name
        .clone()
        .unwrap_or_else(|| "Sheet1".into());
    let response = match prod
        .fetch_csv(&sheet_id, sheet_gid.as_deref(), &sheet_name)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return WorksheetConnectionReport {
                ok: false,
                sheet_name: Some(sheet_name),
                url: None,
                http_status: None,
                row_count: None,
                header_valid: false,
                message: e.to_string(),
                detail: None,
            }
        }
    };
    let (records, issues, _table_rows) =
        crate::ingestion::sheet_parser::parse_csv(&response.body, &[]);
    let header_valid = response.status.is_success()
        && !issues
            .iter()
            .any(|issue| issue.code == crate::domain::IssueCode::HeaderMismatch);
    let ok = response.status.is_success() && header_valid;
    WorksheetConnectionReport {
        ok,
        sheet_name: Some(sheet_name),
        url: Some(response.url),
        http_status: Some(response.status.as_u16()),
        row_count: Some(records.len()),
        header_valid,
        message: if ok {
            "資料工作表可連線".into()
        } else if !response.status.is_success() {
            response.message.clone()
        } else {
            "已連上 Google，但回應內容不是可解析的 Sheet CSV。".into()
        },
        detail: response.detail.or_else(|| {
            if issues.is_empty() {
                None
            } else {
                Some(
                    issues
                        .into_iter()
                        .map(|issue| {
                            format!("第 {} 列：{}", issue.source_row_number, issue.message_zh_tw)
                        })
                        .collect::<Vec<_>>()
                        .join("；"),
                )
            }
        }),
    }
}

/// 測試單一設定工作表：抓取後檢查 HTTP、header、可衍生性。
async fn test_settings_worksheet(
    fetcher: &dyn GoogleSheetFetcher,
    sheet_id: &str,
    sheet_name: &str,
    label: &str,
    kind: SettingsKind,
) -> WorksheetConnectionReport {
    let response = match fetcher.fetch_csv(sheet_id, None, sheet_name).await {
        Ok(r) => r,
        Err(e) => {
            return WorksheetConnectionReport {
                ok: false,
                sheet_name: Some(sheet_name.into()),
                url: None,
                http_status: None,
                row_count: None,
                header_valid: false,
                message: e.to_string(),
                detail: None,
            }
        }
    };
    if !response.status.is_success() {
        return WorksheetConnectionReport {
            ok: false,
            sheet_name: Some(sheet_name.into()),
            url: Some(response.url),
            http_status: Some(response.status.as_u16()),
            row_count: None,
            header_valid: false,
            message: format!("「{label}」工作表讀取失敗（HTTP {}）。", response.status),
            detail: response.detail,
        };
    }
    // 驗證可衍生性：用空白關鍵字/標準值暫試 parse（standards 需自洽）。
    // 此處僅檢查 header 與基本格式；完整衍生驗證在實際 sync/dashboard 時。
    let body = &response.body;
    let (header_valid, row_count, message) = match kind {
        SettingsKind::Keywords => {
            let rows = count_csv_rows(body);
            let valid = header_starts_with(body, "事件關鍵字");
            (
                valid,
                rows,
                if valid {
                    "「事件關鍵字設定」可連線".into()
                } else {
                    "標頭應為「事件關鍵字」。".into()
                },
            )
        }
        SettingsKind::Standards => {
            let rows = count_csv_rows(body);
            let valid = header_starts_with(body, "事件,血糖下限,血糖上限");
            (
                valid,
                rows,
                if valid {
                    "「血糖標準值設定」可連線".into()
                } else {
                    "標頭應為「事件,血糖下限,血糖上限」。".into()
                },
            )
        }
    };
    WorksheetConnectionReport {
        ok: response.status.is_success() && header_valid,
        sheet_name: Some(sheet_name.into()),
        url: Some(response.url),
        http_status: Some(response.status.as_u16()),
        row_count: Some(row_count),
        header_valid,
        message,
        detail: response.detail,
    }
}

/// 計算 CSV 資料列數（不含 header）。
fn count_csv_rows(text: &str) -> usize {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let mut iter = reader.records();
    let _ = iter.next(); // header
    iter.filter_map(|r| r.ok()).count()
}

/// 檢查 CSV 首列是否以預期 header 開頭。
fn header_starts_with(text: &str, expected: &str) -> bool {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let header = reader
        .records()
        .next()
        .and_then(|r| r.ok())
        .map(|rec| rec.iter().map(String::from).collect::<Vec<_>>())
        .unwrap_or_default();
    let expected_cols: Vec<&str> = expected.split(',').collect();
    header.len() == expected_cols.len()
        && header.iter().zip(expected_cols.iter()).all(|(h, e)| h == e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::ingestion::settings_loader::parse_sheet_settings;

    fn temp_state() -> super::ApiState {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "glucose-dashboard-config-test-{}-{}.json",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        super::ApiState {
            config: crate::config::store::ConfigStore::from_path(path),
            records_cache: Arc::new(RwLock::new(None)),
            sheet_fetcher: Arc::new(crate::ingestion::sync_service::ReqwestGoogleSheetFetcher),
        }
    }

    #[tokio::test]
    async fn configure_persists_worksheet_names_and_clears_cache() {
        let state = temp_state();
        let saved = super::configure(
            axum::extract::State(state.clone()),
            axum::Json(super::ConfigureRequest {
                sheet_id: "ABC".into(),
                sheet_name: Some("Sheet1".into()),
                fixture_path: None,
                event_keywords_sheet_name: Some("我的關鍵字".into()),
                glucose_standards_sheet_name: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            saved.event_keywords_sheet_name.as_deref(),
            Some("我的關鍵字")
        );
        // 空白 → 預設常數。
        assert_eq!(
            saved.glucose_standards_sheet_name.as_deref(),
            Some(crate::config::model::DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME)
        );
        // 快取應被清空。
        assert!(state.records_cache.read().await.is_none());
    }

    #[tokio::test]
    async fn status_returns_empty_vectors_before_first_sync() {
        let state = temp_state();
        let status = super::status(axum::extract::State(state)).await;
        assert!(status.0.custom_events.is_empty());
        assert!(status.0.event_thresholds.is_empty());
    }

    #[test]
    fn header_starts_with_detects_standards_header() {
        let csv = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n";
        assert!(header_starts_with(csv, "事件,血糖下限,血糖上限"));
        assert!(!header_starts_with(csv, "事件關鍵字"));
    }

    #[test]
    fn count_csv_rows_excludes_header() {
        let csv = "事件關鍵字\n飲食測試\n運動後\n";
        assert_eq!(count_csv_rows(csv), 2);
    }

    #[test]
    fn parse_sheet_settings_rejects_missing_builtin() {
        let keywords = "事件關鍵字\n飲食測試\n";
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n";
        assert!(parse_sheet_settings(keywords, bad).is_err());
    }

    // 保留對 AppError::Invalid 的基本驗證（configure 拒絕空 ID）。
    #[tokio::test]
    async fn configure_rejects_empty_sheet_id() {
        let state = temp_state();
        let result = super::configure(
            axum::extract::State(state),
            axum::Json(super::ConfigureRequest {
                sheet_id: "   ".into(),
                sheet_name: None,
                fixture_path: None,
                event_keywords_sheet_name: None,
                glucose_standards_sheet_name: None,
            }),
        )
        .await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }
}
