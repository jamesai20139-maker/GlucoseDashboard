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
    domain::{AnalysisSelection, DashboardTableRow, Event, GlucoseRecord, Period},
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
    pub table_rows: Vec<DashboardTableRow>,
    pub issues: Vec<crate::domain::DataQualityIssue>,
    pub status: &'static str,
    pub last_successful_sync_at: Option<String>,
}

async fn load_records(
    state: &ApiState,
) -> Result<
    (
        Vec<GlucoseRecord>,
        Vec<DashboardTableRow>,
        Vec<crate::domain::DataQualityIssue>,
        Option<String>,
    ),
    AppError,
> {
    let config = state.config.load();
    let service = SyncService {
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: config.fixture_path.map(Into::into),
    };
    let (records, issues, table_rows) = service.load().await?;
    Ok((records, table_rows, issues, config.last_successful_sync_at))
}

/// 篩選表格顯示列。有效列套用 period/event/search（與 `selection::filter` 同邏輯）；
/// 錯誤列（任一 parsed 欄位為 None）只套用 search，不受 period/event 影響，
/// 避免錯誤列因缺少有效日期/事件而消失。
fn filter_table_rows(rows: &[DashboardTableRow], selection: &AnalysisSelection) -> Vec<DashboardTableRow> {
    let needle = selection.search.as_ref().map(|text| text.to_lowercase());
    rows.iter()
        .filter(|row| {
            let valid = row.parsed_measured_at.is_some() && row.parsed_event.is_some();
            if valid {
                // 有效列：套 period/event
                let date_ok = row
                    .parsed_measured_at
                    .map(|dt| selection.period.contains(dt))
                    .unwrap_or(false);
                let event_ok = selection
                    .event
                    .as_ref()
                    .map(|event| row.parsed_event.as_ref() == Some(event))
                    .unwrap_or(true);
                date_ok && event_ok
            } else {
                // 錯誤列：永遠保留（不受 period/event 影響）
                true
            }
        })
        .filter(|row| match &needle {
            Some(text) => {
                row.measured_at.as_deref().unwrap_or("").to_lowercase().contains(text)
                    || row.event.as_deref().unwrap_or("").to_lowercase().contains(text)
                    || row.glucose_mg_dl.as_deref().unwrap_or("").to_lowercase().contains(text)
                    || row.remark_1.to_lowercase().contains(text)
                    || row.remark_2.to_lowercase().contains(text)
            }
            None => true,
        })
        .cloned()
        .collect()
}

pub async fn dashboard(
    State(state): State<ApiState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, AppError> {
    let chosen = selection(&query);
    let (records, table_rows, issues, last_sync) = load_records(&state).await?;
    let filtered = selection::filter(&records, &chosen);
    let summary = summary::calculate(&filtered);
    let table_filtered = filter_table_rows(&table_rows, &chosen);
    Ok(Json(DashboardResponse {
        selection: chosen,
        summary,
        records: filtered,
        table_rows: table_filtered,
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
    let (records, _table_rows, _issues, _last_sync) = load_records(&state).await?;
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
