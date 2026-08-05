use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::service,
    diagnostics::checks,
    errors::AppError,
};

#[derive(Serialize)]
pub struct ConfigStatus {
    pub configured: bool,
    pub credential_store: &'static str,
    pub schema_version: u32,
    pub sheet_id: Option<String>,
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
