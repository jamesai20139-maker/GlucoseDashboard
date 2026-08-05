use axum::{
    extract::{Query, State},
    http::{header, HeaderValue},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};

use super::router::ApiState;
use crate::{
    analysis::{selection, summary},
    config::store::ConfigStore,
    domain::{AnalysisSelection, Event, GlucoseRecord, Period},
    errors::AppError,
    ingestion::sync_service::SyncService,
};

#[derive(Deserialize, Default)]
pub struct DashboardQuery {
    pub event: Option<String>,
    pub search: Option<String>,
    pub period: Option<String>,
}

fn selection(query: &DashboardQuery) -> AnalysisSelection {
    let period = match query.period.as_deref() {
        Some("day") => Period::Day,
        Some("week") => Period::Week,
        Some("month") => Period::Month,
        Some("quarter") => Period::Quarter,
        _ => Period::Custom {
            start: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            end: chrono::NaiveDate::from_ymd_opt(2100, 1, 1).unwrap(),
        },
    };
    AnalysisSelection {
        period,
        event: query.event.as_deref().and_then(Event::parse),
        search: query.search.clone(),
    }
}

#[derive(Serialize)]
pub struct DashboardResponse {
    pub selection: AnalysisSelection,
    pub summary: crate::domain::AnalysisSummary,
    pub records: Vec<GlucoseRecord>,
    pub issues: Vec<crate::domain::DataQualityIssue>,
    pub status: &'static str,
    pub last_successful_sync_at: Option<String>,
}

fn load_records(
    state: &ApiState,
) -> Result<
    (
        Vec<GlucoseRecord>,
        Vec<crate::domain::DataQualityIssue>,
        Option<String>,
    ),
    AppError,
> {
    let config = state.config.load();
    let service = SyncService {
        fixture_path: config.fixture_path.map(Into::into),
    };
    let (records, issues) = service.load()?;
    Ok((records, issues, config.last_successful_sync_at))
}

pub async fn dashboard(
    State(state): State<ApiState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, AppError> {
    let chosen = selection(&query);
    let (records, issues, last_sync) = load_records(&state)?;
    let filtered = selection::filter(&records, &chosen);
    let summary = summary::calculate(&filtered);
    Ok(Json(DashboardResponse {
        selection: chosen,
        summary,
        records: filtered,
        issues,
        status: "succeeded",
        last_successful_sync_at: last_sync,
    }))
}

pub async fn export_csv(
    State(state): State<ApiState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Response, AppError> {
    let chosen = selection(&query);
    let (records, _, _) = load_records(&state)?;
    let records = selection::filter(&records, &chosen);
    let mut csv = String::from("血糖量測日期時間,事件,量測血糖值(mg/dl),備註1,備註2\n");
    for record in records {
        csv.push_str(&format!(
            "\"{}\",\"{}\",{},\"{}\",\"{}\"\n",
            record.measured_at.format("%Y/%m/%d %H:%M"),
            record.event.label_zh_tw(),
            record.glucose_mg_dl,
            record.remark_1.replace('"', "\"\""),
            record.remark_2.replace('"', "\"\"")
        ));
    }
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv; charset=utf-8"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=glucose-records.csv"),
        )
        .body(csv.into())
        .map_err(|error| AppError::Internal(error.to_string()))
}
