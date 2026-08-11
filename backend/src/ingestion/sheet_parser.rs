use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use crate::domain::{
    CustomEvent, DashboardTableRow, DataQualityIssue, Event, GlucoseRecord, IssueCode,
    IssueSeverity,
};

pub const HEADERS: [&str; 5] = [
    "血糖量測日期時間",
    "事件",
    "量測血糖值(mg/dl)",
    "備註1",
    "備註2",
];

fn issue(row: usize, severity: IssueSeverity, code: IssueCode, message: &str) -> DataQualityIssue {
    DataQualityIssue {
        source_row_number: row,
        severity,
        code,
        message_zh_tw: message.into(),
    }
}

pub fn parse_date(value: &str) -> Option<DateTime<Utc>> {
    let trimmed = value.trim();
    // 純數字 24 小時制格式（分鐘或帶秒）。chrono 對數字欄位寬度寬鬆，
    // 不補零的 "2026/8/7 14:30" 亦可解析。
    let numeric_formats = [
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ];
    if let Some(dt) = numeric_formats
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(trimmed, format).ok())
    {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    }
    // Google Sheets zh-TW 本地化匯出格式，例如 "2026/8/7 下午 12:04:00"：
    // 日期以 / 或 - 分隔且不補零、12 小時制、含「上午/下午」中文、可能帶秒。
    // chrono 的 %p 僅認英文 AM/PM，無法直接處理中文，故手動解析。
    parse_localized_zh_tw(trimmed).map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}

/// 解析 Google Sheets zh-TW 本地化日期時間字串。
fn parse_localized_zh_tw(value: &str) -> Option<NaiveDateTime> {
    let value = value.trim();
    // 以第一個空白切出日期段與剩餘時間段（日期段不含空白）。
    let (date_part, rest) = value.split_once(' ')?;
    let rest = rest.trim();
    // 偵測「上午/下午」 meridiem；不存在則視為 24 小時制。
    let (time_part, meridiem) = if let Some(time) = rest.strip_prefix("上午") {
        (time.trim(), Some(false))
    } else if let Some(time) = rest.strip_prefix("下午") {
        (time.trim(), Some(true))
    } else {
        (rest, None)
    };
    let (year, month, day) = parse_date_components(date_part)?;
    let (hour, minute, second) = parse_time_components(time_part)?;
    let hour24 = match meridiem {
        None => {
            if hour > 23 {
                return None;
            }
            hour
        }
        Some(is_pm) => to_24h(hour, is_pm)?,
    };
    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour24, minute, second)
}

/// 解析日期段 `2026/8/7` 或 `2026-8-7`（不補零，/ 或 - 分隔）。
fn parse_date_components(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s.split(['/', '-']).collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    Some((year, month, day))
}

/// 解析時間段 `12:04:00` 或 `6:30`（不補零，秒數可有可無）。
/// 僅檢查分秒範圍；時的範圍由呼叫端依 12/24 小時制各自驗證。
fn parse_time_components(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;
    let second = parts
        .get(2)
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);
    if minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second))
}

/// 將 12 小時制時數轉為 24 小時制。`hour12` 須為 1..=12。
fn to_24h(hour12: u32, is_pm: bool) -> Option<u32> {
    if !(1..=12).contains(&hour12) {
        return None;
    }
    Some(match (hour12, is_pm) {
        (12, false) => 0, // 上午 12 → 半夜 0 時
        (12, true) => 12, // 下午 12 → 中午 12 時
        (h, false) => h,
        (h, true) => h + 12,
    })
}

/// 判斷血糖值是否有效（整數且在 20–600）。
fn valid_glucose(raw: &str) -> Option<i32> {
    let glucose = raw.trim().parse::<i32>().ok()?;
    if (20..=600).contains(&glucose) {
        Some(glucose)
    } else {
        None
    }
}

pub fn parse_rows(
    headers: &[String],
    rows: &[Vec<String>],
    custom: &[CustomEvent],
) -> (
    Vec<GlucoseRecord>,
    Vec<DataQualityIssue>,
    Vec<DashboardTableRow>,
) {
    let mut records = Vec::new();
    let mut issues = Vec::new();
    let mut table_rows = Vec::new();
    if headers.iter().map(String::as_str).collect::<Vec<_>>() != HEADERS {
        issues.push(issue(
            1,
            IssueSeverity::Error,
            IssueCode::HeaderMismatch,
            "Google Sheet 欄位標題不符合規定，請檢查名稱與順序。",
        ));
        return (records, issues, table_rows);
    }
    for (index, row) in rows.iter().enumerate() {
        let source_row = index + 2;
        let raw_date = row.first().map(|s| s.trim()).unwrap_or("");
        let raw_event = row.get(1).map(|s| s.trim()).unwrap_or("");
        let raw_glucose = row.get(2).map(|s| s.trim()).unwrap_or("");
        let remark_1 = row.get(3).cloned().unwrap_or_default();
        let remark_2 = row.get(4).cloned().unwrap_or_default();

        // 表格顯示列：每一列都產出，有效欄位放顯示值、無效為 None。
        let parsed_measured_at = parse_date(raw_date);
        let parsed_event = Event::parse(raw_event, custom);
        let parsed_glucose = valid_glucose(raw_glucose);
        table_rows.push(DashboardTableRow {
            source_row_number: source_row,
            measured_at: parsed_measured_at.map(|dt| dt.format("%Y/%m/%d %H:%M").to_string()),
            event: parsed_event.as_ref().map(|e| e.label_zh_tw()),
            glucose_mg_dl: parsed_glucose.map(|g| g.to_string()),
            remark_1: remark_1.clone(),
            remark_2: remark_2.clone(),
            parsed_measured_at,
            parsed_event,
        });

        // 既有驗證邏輯：缺欄位、日期、事件、血糖依序檢查，有效才進 records。
        if row.len() < 3 || row[..3].iter().any(|value| value.trim().is_empty()) {
            issues.push(issue(
                source_row,
                IssueSeverity::Warning,
                IssueCode::MissingField,
                "此筆資料缺少日期、事件或血糖值，已略過。",
            ));
            continue;
        }
        let Some(measured_at) = parse_date(&row[0]) else {
            issues.push(issue(
                source_row,
                IssueSeverity::Error,
                IssueCode::InvalidDate,
                "日期格式無法解析，已略過此筆資料。",
            ));
            continue;
        };
        let Some(event) = Event::parse(&row[1], custom) else {
            issues.push(issue(
                source_row,
                IssueSeverity::Warning,
                IssueCode::UnknownEvent,
                "未知事件不會參與統計。",
            ));
            continue;
        };
        let Some(glucose) = valid_glucose(&row[2]) else {
            issues.push(issue(
                source_row,
                IssueSeverity::Error,
                IssueCode::InvalidGlucose,
                "血糖值不是有效整數，已略過。",
            ));
            continue;
        };
        records.push(GlucoseRecord {
            source_row_number: source_row,
            measured_at,
            event,
            glucose_mg_dl: glucose,
            remark_1,
            remark_2,
        });
    }
    (records, issues, table_rows)
}

pub fn parse_csv(
    text: &str,
    custom: &[CustomEvent],
) -> (
    Vec<GlucoseRecord>,
    Vec<DataQualityIssue>,
    Vec<DashboardTableRow>,
) {
    // 使用 csv crate 正確解析 RFC 4180 CSV，避免備註欄位內含逗號或引號時
    // 被簡單 split(',') 拆錯欄位（例如「早餐,2顆藥」被切成兩欄）。
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let mut rows_iter = reader.records();
    let headers = rows_iter
        .next()
        .and_then(|result| result.ok())
        .map(|record| record.iter().map(String::from).collect::<Vec<_>>())
        .unwrap_or_default();
    let rows = rows_iter
        .filter_map(|result| result.ok())
        .map(|record| record.iter().map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    parse_rows(&headers, &rows, custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers() -> Vec<String> {
        HEADERS.iter().map(|value| (*value).to_string()).collect()
    }

    fn custom(label: &str, low: i32, high: i32) -> CustomEvent {
        CustomEvent {
            label: label.into(),
            low_threshold: low,
            high_threshold: high,
        }
    }

    #[test]
    fn accepts_supported_dates_and_context_rules() {
        let rows = vec![
            vec!["2026/07/07 06:30".into(), "空腹血糖".into(), "99".into()],
            vec!["2026-07-07 12:30".into(), "午餐前".into(), "100".into()],
            vec!["2026/07/07 14:30".into(), "午餐後".into(), "140".into()],
        ];
        let (records, issues, table_rows) = parse_rows(&headers(), &rows, &[]);
        assert_eq!(records.len(), 3);
        assert!(issues.is_empty());
        assert_eq!(table_rows.len(), 3);
        // 表格事件欄為中文
        assert_eq!(table_rows[0].event.as_deref(), Some("空腹血糖"));
        assert_eq!(
            records[0].classify(),
            crate::domain::Classification::InRange
        );
        assert_eq!(records[2].classify(), crate::domain::Classification::High);
    }

    #[test]
    fn excludes_invalid_rows_but_keeps_valid_rows() {
        let rows = vec![
            vec!["bad".into(), "空腹血糖".into(), "90".into()],
            vec!["2026/07/07 06:30".into(), "未知".into(), "90".into()],
            vec!["2026/07/07 06:30".into(), "空腹血糖".into(), "90".into()],
        ];
        let (records, issues, table_rows) = parse_rows(&headers(), &rows, &[]);
        assert_eq!(records.len(), 1);
        assert_eq!(issues.len(), 2);
        // 表格仍含全部 3 列
        assert_eq!(table_rows.len(), 3);
        // 錯誤列的對應欄位為 None
        assert!(table_rows[0].measured_at.is_none());
        assert!(table_rows[1].event.is_none());
        // 有效列欄位齊全
        assert!(table_rows[2].measured_at.is_some());
        assert_eq!(table_rows[2].event.as_deref(), Some("空腹血糖"));
    }

    #[test]
    fn custom_event_rows_join_stats_and_classify_by_thresholds() {
        let defs = [custom("運動後", 70, 139)];
        let rows = vec![
            vec!["2026/07/07 07:00".into(), "運動後".into(), "69".into()],
            vec!["2026/07/07 08:00".into(), "運動後".into(), "100".into()],
            vec!["2026/07/07 09:00".into(), "運動後".into(), "140".into()],
        ];
        let (records, issues, table_rows) = parse_rows(&headers(), &rows, &defs);
        // 三列皆為有效自訂事件，進入統計、無問題。
        assert_eq!(records.len(), 3);
        assert!(issues.is_empty());
        assert_eq!(table_rows.len(), 3);
        // 表格事件欄顯示自訂標籤。
        assert_eq!(table_rows[0].event.as_deref(), Some("運動後"));
        // 依自訂閾值分類：69 偏低、100 正常、140 偏高。
        assert_eq!(records[0].classify(), crate::domain::Classification::Low);
        assert_eq!(
            records[1].classify(),
            crate::domain::Classification::InRange
        );
        assert_eq!(records[2].classify(), crate::domain::Classification::High);
    }

    #[test]
    fn remarks_with_commas_stay_in_correct_column() {
        // 備註欄位內含逗號時，必須整段留在同一欄，不能被拆到備註2。
        let csv_text = "血糖量測日期時間,事件,量測血糖值(mg/dl),備註1,備註2\n\
            2026/07/07 06:30,空腹血糖,99,\"早餐,2顆藥\",\n\
            2026/07/07 12:30,午餐前,100,\"飯前,半顆\",\"飯後,完整\"\n";
        let (records, _issues, table_rows) = parse_csv(csv_text, &[]);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].remark_1, "早餐,2顆藥");
        assert_eq!(records[0].remark_2, "");
        assert_eq!(records[1].remark_1, "飯前,半顆");
        assert_eq!(records[1].remark_2, "飯後,完整");
        assert_eq!(table_rows[0].remark_1, "早餐,2顆藥");
        assert_eq!(table_rows[1].remark_2, "飯後,完整");
    }

    #[test]
    fn parses_google_sheets_zh_tw_localized_dates() {
        // Google Sheets zh-TW 匯出格式：不補零日期、12 小時制、含「上午/下午」、帶秒。
        // 上午 7:11:00 → 07:11:00 UTC；下午 12:04:00 → 12:04:00 UTC；
        // 上午 12:00:00 → 半夜 0 時。
        let rows = vec![
            vec![
                "2026/8/7 上午 7:11:00".into(),
                "空腹血糖".into(),
                "90".into(),
            ],
            vec![
                "2026/8/7 下午 12:04:00".into(),
                "午餐後".into(),
                "140".into(),
            ],
            vec!["2026/8/7 上午 12:00:00".into(), "睡前".into(), "100".into()],
        ];
        let (records, issues, _table_rows) = parse_rows(&headers(), &rows, &[]);
        assert_eq!(records.len(), 3);
        assert!(issues.is_empty());
        assert_eq!(
            records[0].measured_at.format("%H:%M:%S").to_string(),
            "07:11:00"
        );
        assert_eq!(
            records[1].measured_at.format("%H:%M:%S").to_string(),
            "12:04:00"
        );
        assert_eq!(
            records[2].measured_at.format("%H:%M:%S").to_string(),
            "00:00:00"
        );
    }

    #[test]
    fn parses_numeric_formats_with_seconds_and_unpadded() {
        // 帶秒純數字格式（/ 與 - 分隔）與不補零日期皆可解析。
        let cases = [
            ("2026/8/7 14:30:00", "14:30:00"),
            ("2026-8-7 6:5:9", "06:05:09"),
            ("2026/8/7 上午 7:11:00", "07:11:00"),
            ("2026-8-7 下午 11:59:00", "23:59:00"),
        ];
        for (input, expected) in cases {
            let dt = parse_date(input).unwrap_or_else(|| panic!("應可解析 {input:?}"));
            assert_eq!(
                dt.format("%H:%M:%S").to_string(),
                expected,
                "input={input:?}"
            );
        }
    }

    #[test]
    fn invalid_meridiem_hours_produce_no_date() {
        // 12 小時制時數須在 1..=12 之外為無效；24 小時制時數須 <=23。
        // 解析失敗 → 表格 measured_at 為 None、列入 issue，而非顯示錯誤值。
        let rows = vec![
            vec!["2026/8/7 下午 0:00".into(), "空腹血糖".into(), "90".into()],
            vec!["2026/8/7 下午 13:00".into(), "空腹血糖".into(), "90".into()],
            vec!["2026/8/7 25:00".into(), "空腹血糖".into(), "90".into()],
        ];
        let (records, issues, table_rows) = parse_rows(&headers(), &rows, &[]);
        assert_eq!(records.len(), 0);
        assert_eq!(issues.len(), 3);
        assert!(table_rows.iter().all(|r| r.measured_at.is_none()));
        assert!(issues
            .iter()
            .all(|i| i.code == crate::domain::IssueCode::InvalidDate));
    }

    #[test]
    fn custom_event_label_not_in_config_is_unknown_event() {
        // 自訂清單不含「運動後」，該列應回到未知事件行為（排除統計、表格 None）。
        let rows = vec![
            vec!["2026/07/07 07:00".into(), "運動後".into(), "100".into()],
            vec!["2026/07/07 08:00".into(), "空腹血糖".into(), "90".into()],
        ];
        let (records, issues, table_rows) = parse_rows(&headers(), &rows, &[]);
        assert_eq!(records.len(), 1);
        assert_eq!(issues.len(), 1);
        assert!(table_rows[0].event.is_none());
        assert_eq!(table_rows[1].event.as_deref(), Some("空腹血糖"));
    }
}
