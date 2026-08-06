use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use super::{config_routes, dashboard_routes, sync_routes};
use crate::config::store::ConfigStore;

#[derive(Clone)]
pub struct ApiState {
    pub config: ConfigStore,
}

pub fn build_router(config: ConfigStore) -> Router {
    let state = ApiState { config };
    Router::new()
        .route("/api/health", get(|| async { "{\"status\":\"ok\"}" }))
        .route("/api/config/status", get(config_routes::status))
        .route("/api/configure", post(config_routes::configure))
        .route("/api/config/test-connection", get(config_routes::test_connection))
        .route("/api/sync", post(sync_routes::sync))
        .route("/api/dashboard", get(dashboard_routes::dashboard))
        .route("/api/records/export.csv", get(dashboard_routes::export_csv))
        .route("/api/diagnostics", get(config_routes::diagnostics))
        .with_state(state)
        .fallback_service(ServeDir::new("frontend/dist").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
