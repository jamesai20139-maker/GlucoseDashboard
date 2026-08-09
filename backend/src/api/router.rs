use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use super::{config_routes, dashboard_routes, sync_routes};
use crate::{
    config::{model::LocalConfiguration, store::ConfigStore},
    domain::{DashboardTableRow, DataQualityIssue, GlucoseRecord},
};

/// 進程記憶體暫存：存放已抓取並解析的 Sheet 紀錄。不寫磁碟、重啟即清空，
/// 符合憲法「ephemeral analysis」要求。以設定簽章為 key，設定變更即失效。
#[derive(Debug, Clone)]
pub struct RecordsCache {
    pub records: Vec<GlucoseRecord>,
    pub table_rows: Vec<DashboardTableRow>,
    pub issues: Vec<DataQualityIssue>,
    pub fetched_at: DateTime<Utc>,
    pub source_signature: String,
}

#[derive(Clone)]
pub struct ApiState {
    pub config: ConfigStore,
    pub records_cache: Arc<RwLock<Option<RecordsCache>>>,
}

/// 由目前設定算出唯一簽章：sheet_id|gid|name 或 fixture_path。設定一變簽章即不同。
pub fn source_signature(config: &LocalConfiguration) -> String {
    if config
        .sheet_id
        .as_ref()
        .map(|id| !id.trim().is_empty())
        .unwrap_or(false)
    {
        format!(
            "sheet|{}|{}|{}",
            config.sheet_id.clone().unwrap_or_default(),
            config.sheet_gid.clone().unwrap_or_default(),
            config.sheet_name.clone().unwrap_or_default()
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
    };
    Router::new()
        .route("/api/health", get(|| async { "{\"status\":\"ok\"}" }))
        .route("/api/config/status", get(config_routes::status))
        .route("/api/configure", post(config_routes::configure))
        .route("/api/custom-events", post(config_routes::add_custom_event))
        .route(
            "/api/custom-events/:label",
            axum::routing::delete(config_routes::delete_custom_event),
        )
        .route(
            "/api/config/test-connection",
            get(config_routes::test_connection),
        )
        .route("/api/sync", post(sync_routes::sync))
        .route("/api/dashboard", get(dashboard_routes::dashboard))
        .route("/api/records/export.csv", get(dashboard_routes::export_csv))
        .route("/api/diagnostics", get(config_routes::diagnostics))
        .with_state(state)
        .fallback_service(ServeDir::new("frontend/dist").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::source_signature;
    use crate::config::model::LocalConfiguration;

    fn config(sheet_id: Option<&str>, fixture: Option<&str>) -> LocalConfiguration {
        LocalConfiguration {
            schema_version: 2,
            sheet_id: sheet_id.map(str::to_string),
            sheet_gid: None,
            sheet_name: Some("Sheet1".into()),
            fixture_path: fixture.map(str::to_string),
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
        }
    }

    #[test]
    fn signature_prefers_sheet_id() {
        let sig = source_signature(&config(Some("ABC123"), Some("/tmp/x.csv")));
        assert_eq!(sig, "sheet|ABC123||Sheet1");
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
