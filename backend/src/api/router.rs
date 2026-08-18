use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use rust_embed::Embed;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::{config_routes, dashboard_routes, sync_routes};
use crate::{
    config::{
        model::{EventThreshold, LocalConfiguration},
        store::ConfigStore,
    },
    domain::{CustomEvent, DashboardTableRow, DataQualityIssue, GlucoseRecord},
    ingestion::sync_service::{GoogleSheetFetcher, ReqwestGoogleSheetFetcher},
};

/// 編譯期嵌入的前端資產（`frontend/dist`）。產出單一 exe 時不再依賴磁碟上的
/// `frontend/dist` 目錄或當前工作目錄，符合「拷貝到任意目錄即可啟動」的安裝需求。
/// 開發時 `npm --prefix frontend run dev`（Vite :5173）走 proxy，不經此 fallback。
#[derive(Embed)]
#[folder = "../frontend/dist/"]
struct FrontendAsset;

/// 進程記憶體暫存：存放已抓取並解析的 Sheet 紀錄，以及當下衍生的設定
/// （事件關鍵字、血糖標準值）。不寫磁碟、重啟即清空，符合憲法「ephemeral
/// analysis」要求。以設定簽章為 key，設定變更即失效。
#[derive(Debug, Clone)]
pub struct RecordsCache {
    pub records: Vec<GlucoseRecord>,
    pub table_rows: Vec<DashboardTableRow>,
    pub issues: Vec<DataQualityIssue>,
    pub custom_events: Vec<CustomEvent>,
    pub event_thresholds: Vec<EventThreshold>,
    pub fetched_at: DateTime<Utc>,
    pub source_signature: String,
}

#[derive(Clone)]
pub struct ApiState {
    pub config: ConfigStore,
    pub records_cache: Arc<RwLock<Option<RecordsCache>>>,
    pub sheet_fetcher: Arc<dyn GoogleSheetFetcher>,
}

/// 由目前設定算出唯一簽章：sheet_id|gid|name|兩工作表名 或 fixture_path。
/// 設定一變簽章即不同。工作表名變更可偵測；工作表內容變更靠快取比對
/// `custom_events`/`event_thresholds`（見 dashboard_routes）。
pub fn source_signature(config: &LocalConfiguration) -> String {
    if config
        .sheet_id
        .as_ref()
        .map(|id| !id.trim().is_empty())
        .unwrap_or(false)
    {
        format!(
            "sheet|{}|{}|{}|{}|{}",
            config.sheet_id.clone().unwrap_or_default(),
            config.sheet_gid.clone().unwrap_or_default(),
            config.sheet_name.clone().unwrap_or_default(),
            config.event_keywords_sheet_name.clone().unwrap_or_default(),
            config
                .glucose_standards_sheet_name
                .clone()
                .unwrap_or_default(),
        )
    } else if let Some(path) = &config.fixture_path {
        format!("fixture|{}", path)
    } else {
        "none".into()
    }
}

pub fn build_router(config: ConfigStore) -> Router {
    let state = ApiState {
        config,
        records_cache: Arc::new(RwLock::new(None)),
        sheet_fetcher: Arc::new(ReqwestGoogleSheetFetcher),
    };
    // 開發友好：若當前工作目錄下有 `frontend/dist`（如 `make run` 先建好），
    // 走磁碟檔案，方便前端改完即重 build 生效；否則用嵌入資產，讓安裝版
    // 單一 exe 不依賴 CWD 與磁碟 dist 目錄。
    let frontend_fallback = if std::path::Path::new("frontend/dist").is_dir() {
        get(serve_from_disk)
    } else {
        get(serve_embedded)
    };
    Router::new()
        .route("/api/health", get(|| async { "{\"status\":\"ok\"}" }))
        .route("/api/config/status", get(config_routes::status))
        .route("/api/configure", post(config_routes::configure))
        .route(
            "/api/config/test-connection",
            get(config_routes::test_connection),
        )
        .route("/api/sync", post(sync_routes::sync))
        .route("/api/dashboard", get(dashboard_routes::dashboard))
        .route("/api/records/export.csv", get(dashboard_routes::export_csv))
        .route("/api/diagnostics", get(config_routes::diagnostics))
        .with_state(state)
        .fallback(frontend_fallback)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

/// 磁碟模式：從 `frontend/dist` 讀檔。保留開發時前端改完即重 build 生效的便利。
/// 找不到對應檔案時回 `index.html`（SPA fallback）。
async fn serve_from_disk(uri: Uri) -> axum::response::Response {
    let path = uri.path().trim_start_matches('/');
    let file_path = std::path::Path::new("frontend/dist").join(path);
    // 目錄穿越防護：拒絕含 `..` 的路徑。
    if path.contains("..") {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }
    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let mime = mime_guess_from_path(&file_path);
            let mut resp = bytes.into_response();
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            if path.is_empty() || path == "index.html" {
                resp.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            }
            resp
        }
        Err(_) => {
            // SPA fallback：讀 index.html。
            match tokio::fs::read("frontend/dist/index.html").await {
                Ok(bytes) => {
                    let mut resp = bytes.into_response();
                    resp.headers_mut().insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/html; charset=utf-8"),
                    );
                    resp.headers_mut()
                        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
                    resp
                }
                Err(_) => (StatusCode::NOT_FOUND, "frontend not built").into_response(),
            }
        }
    }
}

/// 由副檔名猜測 MIME；磁碟模式用（嵌入模式由 rust-embed 提供 mimetype）。
fn mime_guess_from_path(path: &std::path::Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8".into(),
        Some("js") => "application/javascript; charset=utf-8".into(),
        Some("css") => "text/css; charset=utf-8".into(),
        Some("json") => "application/json; charset=utf-8".into(),
        Some("svg") => "image/svg+xml".into(),
        Some("png") => "image/png".into(),
        Some("ico") => "image/x-icon".into(),
        Some("woff") => "font/woff".into(),
        Some("woff2") => "font/woff2".into(),
        _ => "application/octet-stream".into(),
    }
}

/// 從嵌入資產回應前端檔案。路徑對應 `frontend/dist` 內容；找不到時回傳
/// `index.html`（SPA fallback），讓客戶端路由能在任何路徑重新進入。
async fn serve_embedded(uri: Uri) -> axum::response::Response {
    // 去掉前導 '/'，並處理根路徑。
    let path = uri.path().trim_start_matches('/');
    // SPA：根路徑或無對應資產的路徑一律回 index.html，讓客戶端路由重新進入。
    let asset_path = if path.is_empty() || FrontendAsset::get(path).is_none() {
        "index.html"
    } else {
        path
    };
    match FrontendAsset::get(asset_path) {
        Some(file) => {
            // MIME 來自 rust-embed 內建（mime-guess feature）的 metadata。
            let mime = file.metadata.mimetype();
            let mut resp = file.data.to_vec().into_response();
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            // index.html 不快取，確保更新後立即套用新版；其餘資源可長快取。
            if asset_path == "index.html" {
                resp.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            }
            resp
        }
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::source_signature;
    use crate::config::model::LocalConfiguration;

    fn config(sheet_id: Option<&str>, fixture: Option<&str>) -> LocalConfiguration {
        LocalConfiguration {
            schema_version: 4,
            sheet_id: sheet_id.map(str::to_string),
            sheet_gid: None,
            sheet_name: Some("Sheet1".into()),
            fixture_path: fixture.map(str::to_string),
            credential_reference: None,
            last_successful_sync_at: None,
            event_keywords_sheet_name: Some("事件關鍵字設定".into()),
            glucose_standards_sheet_name: Some("血糖標準值設定".into()),
        }
    }

    #[test]
    fn signature_prefers_sheet_id() {
        let sig = source_signature(&config(Some("ABC123"), Some("/tmp/x.csv")));
        assert_eq!(sig, "sheet|ABC123||Sheet1|事件關鍵字設定|血糖標準值設定");
    }

    #[test]
    fn signature_uses_fixture_when_no_sheet() {
        let sig = source_signature(&config(None, Some("/tmp/valid.csv")));
        assert_eq!(sig, "fixture|/tmp/valid.csv");
    }

    #[test]
    fn signature_changes_when_sheet_id_changes() {
        let a = source_signature(&config(Some("AAA"), None));
        let b = source_signature(&config(Some("BBB"), None));
        assert_ne!(a, b);
    }

    #[test]
    fn signature_none_when_unconfigured() {
        assert_eq!(source_signature(&config(None, None)), "none");
    }
}
