use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;

use super::router::ApiState;
use crate::{config::store::ConfigStore, errors::AppError, ingestion::sync_service::SyncService};

#[derive(Serialize)]
pub struct SyncResponse {
    pub status: &'static str,
    pub records: Vec<crate::domain::GlucoseRecord>,
    pub issues: Vec<crate::domain::DataQualityIssue>,
    pub last_successful_sync_at: Option<String>,
}

pub async fn sync(State(state): State<ApiState>) -> Result<Json<SyncResponse>, AppError> {
    let config = state.config.load();
    let service = SyncService {
        fixture_path: config.fixture_path.clone().map(Into::into),
    };
    let (records, issues) = service.load()?;
    let timestamp = Utc::now().to_rfc3339();
    let mut updated = config;
    updated.last_successful_sync_at = Some(timestamp.clone());
    let _ = state.config.save(&updated);
    Ok(Json(SyncResponse {
        status: "succeeded",
        records,
        issues,
        last_successful_sync_at: Some(timestamp),
    }))
}
