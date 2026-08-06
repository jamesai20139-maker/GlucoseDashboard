use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::service,
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
}

pub async fn status(State(state): State<ApiState>) -> Json<ConfigStatus> {
    let config = state.config.load();
    let store = crate::auth::credential_store::CredentialStore;
    Json(ConfigStatus {
        configured: config.is_configured(),
        credential_store: store.status(),
        schema_version: config.schema_version,
        sheet_id: config.sheet_id,
        sheet_gid: config.sheet_gid,
        sheet_name: config.sheet_name,
        fixture_path: config.fixture_path,
        last_successful_sync_at: config.last_successful_sync_at,
    })
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
    service::configure(
        &state.config,
        request.sheet_id,
        sheet_name,
        request.fixture_path,
    )
    .map(Json)
}

pub async fn diagnostics(State(state): State<ApiState>) -> Json<Vec<checks::CheckResult>> {
    Json(checks::run(&state.config))
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

pub async fn test_connection(State(state): State<ApiState>) -> Result<Json<ConnectionTestResponse>, AppError> {
    let config = state.config.load();
    let service = SyncService {
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: None,
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
