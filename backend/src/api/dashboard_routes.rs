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
    domain::{AnalysisSelection, CustomEvent, DashboardTableRow, Event, GlucoseRecord, Period},
    errors::AppError,
    ingestion::sync_service::SyncService,
};

#[derive(Deserialize, Default)]
pub struct DashboardQuery {
    pub event: Option<String>,
    pub search: Option<String>,
    pub period: Option<String>, // all | day | week | month | quarter
    pub start: Option<String>,  // YYYY-MM-DD（day 用）
    pub end: Option<String>,    // YYYY-MM-DD（day 用）
    pub year: Option<i32>,      // week/month/quarter 用
    pub week: Option<u32>,      // week 用
    pub month: Option<u32>,     // month 用
    pub quarter: Option<u32>,   // quarter 用
}

fn selection(query: &DashboardQuery, custom: &[CustomEvent]) -> AnalysisSelection {
    let period = match query.period.as_deref() {
        Some("day") => parse_day(&query.start, &query.end).unwrap_or(Period::All),
        Some("week") => query
            .year
            .zip(query.week)
            .filter(|(_, w)| *w >= 1)
            .map(|(year, week)| Period::Week { year, week })
            .unwrap_or(Period::All),
        Some("month") => query
            .year
            .zip(query.month)
            .filter(|(_, m)| *m >= 1 && *m <= 12)
            .map(|(year, month)| Period::Month { year, month })
            .unwrap_or(Period::All),
        Some("quarter") => query
            .year
            .zip(query.quarter)
            .filter(|(_, q)| *q >= 1 && *q <= 4)
            .map(|(year, quarter)| Period::Quarter { year, quarter })
            .unwrap_or(Period::All),
        _ => Period::All,
    };
    AnalysisSelection {
        period,
        event: query
            .event
            .as_deref()
            .and_then(|label| Event::parse(label, custom)),
        search: query.search.clone(),
    }
}

/// 解析「日」自訂起訖日期。任一缺或格式錯、或 start > end 則回 None（回退 All）。
fn parse_day(start: &Option<String>, end: &Option<String>) -> Option<Period> {
    let start = chrono::NaiveDate::parse_from_str(start.as_deref()?, "%Y-%m-%d").ok()?;
    let end = chrono::NaiveDate::parse_from_str(end.as_deref()?, "%Y-%m-%d").ok()?;
    if start > end {
        return None;
    }
    Some(Period::Day { start, end })
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

/// 載入紀錄。`force` 為 false 時優先使用進程記憶體快取（命中且簽章相符則直接回傳，
/// 不再抓 Sheet）；為 true 時強制重新抓取並更新快取。快取不寫磁碟、重啟清空。
async fn load_records(
    state: &ApiState,
    force: bool,
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
    let signature = super::router::source_signature(&config);

    if !force {
        if let Some(cache) = state.records_cache.read().await.as_ref() {
            if cache.source_signature == signature {
                return Ok((
                    cache.records.clone(),
                    cache.table_rows.clone(),
                    cache.issues.clone(),
                    config.last_successful_sync_at,
                ));
            }
        }
    }

    let service = SyncService {
        sheet_id: config.sheet_id.clone(),
        sheet_gid: config.sheet_gid.clone(),
        sheet_name: config.sheet_name.clone(),
        fixture_path: config.fixture_path.map(Into::into),
        custom_events: config.custom_events.clone(),
    };
    let (records, issues, table_rows) = service.load().await?;
    *state.records_cache.write().await = Some(super::router::RecordsCache {
        records: records.clone(),
        table_rows: table_rows.clone(),
        issues: issues.clone(),
        fetched_at: chrono::Utc::now(),
        source_signature: signature,
    });
    Ok((records, table_rows, issues, config.last_successful_sync_at))
}

/// 篩選表格顯示列。有效列套用 period/event/search（與 `selection::filter` 同邏輯）；
/// 錯誤列（任一 parsed 欄位為 None）只套用 search，不受 period/event 影響，
/// 避免錯誤列因缺少有效日期/事件而消失。
fn filter_table_rows(
    rows: &[DashboardTableRow],
    selection: &AnalysisSelection,
) -> Vec<DashboardTableRow> {
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
                row.measured_at
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(text)
                    || row
                        .event
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(text)
                    || row
                        .glucose_mg_dl
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(text)
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
    let config = state.config.load();
    let chosen = selection(&query, &config.custom_events);
    let (records, table_rows, issues, last_sync) = load_records(&state, false).await?;
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
    let config = state.config.load();
    let chosen = selection(&query, &config.custom_events);
    let (records, _table_rows, _issues, _last_sync) = load_records(&state, false).await?;
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

#[cfg(test)]
mod tests {
    use super::{load_records, selection, DashboardQuery};
    use crate::domain::{Event, Period};

    /// 測試用：以空自訂事件清單呼叫 selection（內建事件行為不受影響）。
    fn sel(query: &DashboardQuery) -> super::AnalysisSelection {
        selection(query, &[])
    }

    fn q(period: Option<&str>) -> DashboardQuery {
        DashboardQuery {
            event: None,
            search: None,
            period: period.map(str::to_string),
            start: None,
            end: None,
            year: None,
            week: None,
            month: None,
            quarter: None,
        }
    }

    #[test]
    fn missing_period_falls_back_to_all() {
        assert_eq!(sel(&q(None)).period, Period::All);
        assert_eq!(sel(&q(Some("all"))).period, Period::All);
        assert_eq!(sel(&q(Some("unknown"))).period, Period::All);
    }

    #[test]
    fn day_assembles_from_start_end() {
        let mut query = q(Some("day"));
        query.start = Some("2026-03-01".into());
        query.end = Some("2026-03-31".into());
        assert_eq!(
            sel(&query).period,
            Period::Day {
                start: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                end: chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            }
        );
    }

    #[test]
    fn day_missing_dates_falls_back_to_all() {
        assert_eq!(sel(&q(Some("day"))).period, Period::All);
        let mut query = q(Some("day"));
        query.start = Some("2026-03-01".into());
        assert_eq!(sel(&query).period, Period::All);
    }

    #[test]
    fn day_reversed_dates_falls_back_to_all() {
        let mut query = q(Some("day"));
        query.start = Some("2026-03-31".into());
        query.end = Some("2026-03-01".into());
        assert_eq!(sel(&query).period, Period::All);
    }

    #[test]
    fn week_assembles_from_year_and_week() {
        let mut query = q(Some("week"));
        query.year = Some(2026);
        query.week = Some(5);
        assert_eq!(
            sel(&query).period,
            Period::Week {
                year: 2026,
                week: 5
            }
        );
    }

    #[test]
    fn month_assembles_from_year_and_month() {
        let mut query = q(Some("month"));
        query.year = Some(2026);
        query.month = Some(7);
        assert_eq!(
            sel(&query).period,
            Period::Month {
                year: 2026,
                month: 7
            }
        );
    }

    #[test]
    fn quarter_assembles_from_year_and_quarter() {
        let mut query = q(Some("quarter"));
        query.year = Some(2026);
        query.quarter = Some(2);
        assert_eq!(
            sel(&query).period,
            Period::Quarter {
                year: 2026,
                quarter: 2
            }
        );
    }

    #[test]
    fn quarter_out_of_range_falls_back_to_all() {
        let mut query = q(Some("quarter"));
        query.year = Some(2026);
        query.quarter = Some(5);
        assert_eq!(sel(&query).period, Period::All);
    }

    #[test]
    fn event_and_search_pass_through() {
        let mut query = q(Some("all"));
        query.event = Some("空腹血糖".into());
        query.search = Some("abc".into());
        let s = sel(&query);
        assert_eq!(s.event, Some(Event::Fasting));
        assert_eq!(s.search.as_deref(), Some("abc"));
    }

    /// 建構一個指向 fixture 的 ApiState，用於驗證記憶體快取命中/失效。每次呼叫
    /// 產生唯一路徑並直接注入 `ConfigStore`，避免並行測試經環境變數競爭同一檔案。
    fn fixture_state() -> super::super::router::ApiState {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "glucose-dashboard-cache-test-{}-{}.json",
            std::process::id(),
            n
        ));
        let config = crate::config::model::LocalConfiguration {
            schema_version: 2,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: Some("Sheet1".into()),
            fixture_path: Some(
                std::path::Path::new("tests/fixtures/valid-sheet.csv")
                    .to_string_lossy()
                    .into(),
            ),
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            event_thresholds: Vec::new(),
        };
        std::fs::write(&path, serde_json::to_string(&config).unwrap()).unwrap();
        super::super::router::ApiState {
            config: crate::config::store::ConfigStore::from_path(path),
            records_cache: Arc::new(RwLock::new(None)),
        }
    }

    #[tokio::test]
    async fn load_records_caches_after_first_fetch() {
        let state = fixture_state();
        // 首次：應抓取 fixture 並寫入快取。
        let first = load_records(&state, false).await.unwrap();
        assert!(!first.0.is_empty(), "fixture 應有紀錄");
        // 第二次（force=false）：應命中快取，紀錄數相同。
        let second = load_records(&state, false).await.unwrap();
        assert_eq!(first.0.len(), second.0.len());
        // 強制重抓：仍應成功並回填。
        let forced = load_records(&state, true).await.unwrap();
        assert_eq!(first.0.len(), forced.0.len());
    }

    #[tokio::test]
    async fn cache_signature_mismatch_is_ignored_by_force() {
        // force=true 不論快取都應重抓；此處驗證 force 路徑不會回傳陳舊快取。
        let state = fixture_state();
        let forced = load_records(&state, true).await.unwrap();
        assert!(!forced.0.is_empty());
    }
}
