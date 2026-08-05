use chrono::{DateTime, NaiveDateTime, Utc};

use crate::domain::{DataQualityIssue, Event, GlucoseRecord, IssueCode, IssueSeverity};

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
    ["%Y/%m/%d %H:%M", "%Y-%m-%d %H:%M"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value.trim(), format).ok())
        .map(|value| DateTime::<Utc>::from_naive_utc_and_offset(value, Utc))
}

pub fn parse_rows(
    headers: &[String],
    rows: &[Vec<String>],
) -> (Vec<GlucoseRecord>, Vec<DataQualityIssue>) {
    let mut records = Vec::new();
    let mut issues = Vec::new();
    if headers.iter().map(String::as_str).collect::<Vec<_>>() != HEADERS {
        issues.push(issue(
            1,
            IssueSeverity::Error,
            IssueCode::HeaderMismatch,
            "Google Sheet 欄位標題不符合規定，請檢查名稱與順序。",
        ));
        return (records, issues);
    }
    for (index, row) in rows.iter().enumerate() {
        let source_row = index + 2;
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
        let Some(event) = Event::parse(&row[1]) else {
            issues.push(issue(
                source_row,
                IssueSeverity::Warning,
                IssueCode::UnknownEvent,
                "未知事件不會參與統計。",
            ));
            continue;
        };
        let Ok(glucose) = row[2].trim().parse::<i32>() else {
            issues.push(issue(
                source_row,
                IssueSeverity::Error,
                IssueCode::InvalidGlucose,
                "血糖值不是有效整數，已略過。",
            ));
            continue;
        };
        if !(20..=600).contains(&glucose) {
            issues.push(issue(
                source_row,
                IssueSeverity::Error,
                IssueCode::InvalidGlucose,
                "血糖值必須介於 20 至 600 mg/dL，已略過。",
            ));
            continue;
        }
        records.push(GlucoseRecord {
            source_row_number: source_row,
            measured_at,
            event,
            glucose_mg_dl: glucose,
            remark_1: row.get(3).cloned().unwrap_or_default(),
            remark_2: row.get(4).cloned().unwrap_or_default(),
        });
    }
    (records, issues)
}

pub fn parse_csv(text: &str) -> (Vec<GlucoseRecord>, Vec<DataQualityIssue>) {
    let mut lines = text.lines();
    let headers = lines
        .next()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect::<Vec<_>>();
    let rows = lines
        .map(|line| {
            line.split(',')
                .map(|value| value.trim().trim_matches('"').to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    parse_rows(&headers, &rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers() -> Vec<String> {
        HEADERS.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn accepts_supported_dates_and_context_rules() {
        let rows = vec![
            vec!["2026/07/07 06:30".into(), "空腹血糖".into(), "99".into()],
            vec!["2026-07-07 12:30".into(), "午餐前".into(), "100".into()],
            vec!["2026/07/07 14:30".into(), "午餐後".into(), "140".into()],
        ];
        let (records, issues) = parse_rows(&headers(), &rows);
        assert_eq!(records.len(), 3);
        assert!(issues.is_empty());
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
        let (records, issues) = parse_rows(&headers(), &rows);
        assert_eq!(records.len(), 1);
        assert_eq!(issues.len(), 2);
    }
}
