use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    config::{service, store::ConfigStore},
    diagnostics::checks,
    errors::AppError,
};

#[derive(Serialize)]
pub struct ConfigStatus {
    pub configured: bool,
    pub credential_store: &'static str,
    pub sheet_name: Option<String>,
}

pub async fn status(State(state): State<ApiState>) -> Json<ConfigStatus> {
    let config = state.config.load();
    let store = crate::auth::credential_store::CredentialStore;
    Json(ConfigStatus {
        configured: config.is_configured(),
        credential_store: store.status(),
        sheet_name: config.sheet_name,
    })
}

#[derive(Deserialize)]
pub struct ConfigureRequest {
    pub sheet_id: String,
    pub sheet_name: String,
    pub fixture_path: Option<String>,
}

pub async fn configure(
    State(state): State<ApiState>,
    Json(request): Json<ConfigureRequest>,
) -> Result<Json<crate::config::model::LocalConfiguration>, AppError> {
    service::configure(
        &state.config,
        request.sheet_id,
        request.sheet_name,
        request.fixture_path,
    )
    .map(Json)
}

pub async fn diagnostics(State(state): State<ApiState>) -> Json<Vec<checks::CheckResult>> {
    Json(checks::run(&state.config))
}
