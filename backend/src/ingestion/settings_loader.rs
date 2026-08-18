//! 從 Google Sheet 的兩個設定工作表即時衍生事件關鍵字與血糖標準值。
//!
//! - 「事件關鍵字設定」工作表（單欄，header `事件關鍵字`）：自訂事件關鍵字清單。
//! - 「血糖標準值設定」工作表（三欄，header `事件,血糖下限,血糖上限`）：
//!   每個事件的前端顯示標準範圍（含 6 個內建 + 自訂）。
//!
//! 衍生規則：`event_thresholds` 為標準值表逐列；`custom_events` 為關鍵字表
//! 每列查標準值表取 low/high，找不到用 `CUSTOM_EVENT_FALLBACK_*`，與內建
//! 同名略過。兩工作表缺失/空白/格式錯誤 → `AppError::Sync`（阻斷）。
//! 只設本機 CSV（未連結 Sheet）時由呼叫端退回內建預設，不經此模組。

use std::collections::HashSet;

use super::sync_service::GoogleSheetFetcher;
use crate::{
    config::model::{
        builtin_event_thresholds, EventThreshold, BUILTIN_EVENT_LABELS, CUSTOM_EVENT_FALLBACK_HIGH,
        CUSTOM_EVENT_FALLBACK_LOW,
    },
    domain::CustomEvent,
    errors::AppError,
};

/// 從兩個設定工作表衍生的暫時設定。不持久化（符合憲法「ephemeral analysis」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetSettings {
    pub custom_events: Vec<CustomEvent>,
    pub event_thresholds: Vec<EventThreshold>,
}

/// 本機 CSV（未連結 Sheet）時的退回預設：無自訂關鍵字 + 六個內建標準值。
pub fn builtin_fallback_settings() -> SheetSettings {
    SheetSettings {
        custom_events: Vec::new(),
        event_thresholds: builtin_event_thresholds(),
    }
}

/// 關鍵字工作表期望的 header（單欄）。
const KEYWORDS_HEADER: [&str; 1] = ["事件關鍵字"];
/// 標準值工作表期望的 header（三欄）。
const STANDARDS_HEADER: [&str; 3] = ["事件", "血糖下限", "血糖上限"];

/// 純函式：從兩個工作表的 CSV 文字衍生 `SheetSettings`。與網路分離，可單測。
///
/// 驗證規則：
/// - 標準值表 header 必須恰好 `事件,血糖下限,血糖上限`；每列 trim、非空、
///   整數、20..=600、`low < high`；必須含全部六個內建事件；label 不可重複。
/// - 關鍵字表 header 必須恰好 `事件關鍵字`；每列 trim、非空、不重複；與內建
///   同名略過；查標準值表取 low/high，找不到用 fallback 70/139。
/// - 缺表/空白/僅 header/格式錯誤 → `AppError::Sync`（訊息指出哪個工作表/欄位）。
pub fn parse_sheet_settings(
    keywords_csv: &str,
    standards_csv: &str,
) -> Result<SheetSettings, AppError> {
    let standards = parse_standards(standards_csv)?;
    let keywords = parse_keywords(keywords_csv, &standards)?;
    Ok(SheetSettings {
        custom_events: keywords,
        event_thresholds: standards,
    })
}

/// 以標準值表建立 label→EventThreshold 查詢 map。
fn standards_by_label(
    standards: &[EventThreshold],
) -> std::collections::HashMap<&str, &EventThreshold> {
    standards.iter().map(|t| (t.label.as_str(), t)).collect()
}

/// 解析標準值工作表 CSV → `event_thresholds`（列順序即順序）。
fn parse_standards(csv: &str) -> Result<Vec<EventThreshold>, AppError> {
    let (headers, rows) = read_csv(csv).map_err(|m| sync_error("血糖標準值設定", &m))?;
    // header 必須恰好為 `事件,血糖下限,血糖上限`。
    if headers.len() != STANDARDS_HEADER.len()
        || headers
            .iter()
            .zip(STANDARDS_HEADER.iter())
            .any(|(h, expected)| h != expected)
    {
        return Err(sync_error(
            "血糖標準值設定",
            "標頭必須為「事件,血糖下限,血糖上限」三欄。",
        ));
    }
    if rows.is_empty() {
        return Err(sync_error("血糖標準值設定", "工作表無資料列。"));
    }
    let mut thresholds = Vec::with_capacity(rows.len());
    let mut seen: HashSet<String> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let row_no = idx + 2; // 第 1 列為 header，資料從第 2 列起算。
        if row.len() < 3 {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("第 {row_no} 列欄數不足，需三欄（事件,血糖下限,血糖上限）。"),
            ));
        }
        let label = row[0].trim().to_string();
        if label.is_empty() {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("第 {row_no} 列事件名稱不可為空。"),
            ));
        }
        if seen.contains(&label) {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("事件「{label}」重複出現（第 {row_no} 列）。"),
            ));
        }
        seen.insert(label.clone());
        let low = parse_i32(&row[1], "血糖下限", row_no, "血糖標準值設定")?;
        let high = parse_i32(&row[2], "血糖上限", row_no, "血糖標準值設定")?;
        if !(20..=600).contains(&low) || !(20..=600).contains(&high) {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("第 {row_no} 列閾值須介於 20–600。"),
            ));
        }
        if low >= high {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("第 {row_no} 列下限須小於上限。"),
            ));
        }
        thresholds.push(EventThreshold { label, low, high });
    }
    // 必須含全部六個內建事件（保留 UI/篩選不變量）。
    for builtin in BUILTIN_EVENT_LABELS {
        if !thresholds.iter().any(|t| t.label == builtin) {
            return Err(sync_error(
                "血糖標準值設定",
                &format!("缺少內建事件「{builtin}」的標準值。"),
            ));
        }
    }
    Ok(thresholds)
}

/// 解析關鍵字工作表 CSV → `custom_events`（查標準值表取門檻）。
fn parse_keywords(csv: &str, standards: &[EventThreshold]) -> Result<Vec<CustomEvent>, AppError> {
    let (headers, rows) = read_csv(csv).map_err(|m| sync_error("事件關鍵字設定", &m))?;
    if headers.len() != KEYWORDS_HEADER.len()
        || headers
            .iter()
            .zip(KEYWORDS_HEADER.iter())
            .any(|(h, expected)| h != expected)
    {
        return Err(sync_error(
            "事件關鍵字設定",
            "標頭必須為「事件關鍵字」單欄。",
        ));
    }
    // 允許關鍵字表無資料列（代表無自訂事件）。
    let lookup = standards_by_label(standards);
    let mut custom_events = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let row_no = idx + 2;
        if row.is_empty() {
            continue;
        }
        let label = row[0].trim().to_string();
        if label.is_empty() {
            return Err(sync_error(
                "事件關鍵字設定",
                &format!("第 {row_no} 列關鍵字不可為空。"),
            ));
        }
        if seen.contains(&label) {
            return Err(sync_error(
                "事件關鍵字設定",
                &format!("關鍵字「{label}」重複出現（第 {row_no} 列）。"),
            ));
        }
        seen.insert(label.clone());
        // 與內建同名略過（內建已涵蓋）。
        if BUILTIN_EVENT_LABELS.contains(&label.as_str()) {
            continue;
        }
        // 查標準值表取 low/high，找不到用 fallback 70/139。
        let (low, high) = lookup
            .get(label.as_str())
            .map(|t| (t.low, t.high))
            .unwrap_or((CUSTOM_EVENT_FALLBACK_LOW, CUSTOM_EVENT_FALLBACK_HIGH));
        custom_events.push(CustomEvent {
            label,
            low_threshold: low,
            high_threshold: high,
        });
    }
    Ok(custom_events)
}

/// 讀 CSV：回傳 (headers, rows)。沿用 `sheet_parser` 的 csv 設定。
fn read_csv(text: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
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
    Ok((headers, rows))
}

/// 解析整數欄位，失敗訊息含工作表/列號/欄位名。
fn parse_i32(value: &str, field: &str, row_no: usize, sheet: &str) -> Result<i32, AppError> {
    value.trim().parse::<i32>().map_err(|_| {
        sync_error(
            sheet,
            &format!("第 {row_no} 列「{field}」非有效整數：{}", value.trim()),
        )
    })
}

fn sync_error(sheet: &str, message: &str) -> AppError {
    AppError::Sync(format!("「{sheet}」工作表：{message}"))
}

/// 抓取並衍生兩個設定工作表。兩工作表皆以**名稱**抓取（gviz URL），
/// 絕不重用資料表的 gid。生產呼叫端用 `ReqwestGoogleSheetFetcher`。
pub async fn load_sheet_settings_with_fetcher(
    fetcher: &dyn GoogleSheetFetcher,
    sheet_id: &str,
    keywords_sheet_name: &str,
    standards_sheet_name: &str,
) -> Result<SheetSettings, AppError> {
    let keywords =
        fetch_worksheet(fetcher, sheet_id, keywords_sheet_name, "事件關鍵字設定").await?;
    let standards =
        fetch_worksheet(fetcher, sheet_id, standards_sheet_name, "血糖標準值設定").await?;
    parse_sheet_settings(&keywords, &standards)
}

/// 抓取單一設定工作表（by name），回傳 body 文字。非 2xx 或空 body → 阻斷。
async fn fetch_worksheet(
    fetcher: &dyn GoogleSheetFetcher,
    sheet_id: &str,
    sheet_name: &str,
    label: &str,
) -> Result<String, AppError> {
    let response = fetcher.fetch_csv(sheet_id, None, sheet_name).await?;
    if !response.status.is_success() {
        return Err(AppError::Sync(format!(
            "「{label}」工作表讀取失敗（HTTP {}）。",
            response.status
        )));
    }
    let body = response.body;
    if body.trim().is_empty() {
        return Err(AppError::Sync(format!(
            "「{label}」工作表為空白，請確認工作表名稱「{sheet_name}」存在且有資料。"
        )));
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYWORDS_CSV: &str = "事件關鍵字\n飲食測試\n";
    const STANDARDS_CSV: &str = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n睡前,70,140\n飲食測試,70,139\n";

    #[test]
    fn parses_valid_settings() {
        let settings = parse_sheet_settings(KEYWORDS_CSV, STANDARDS_CSV).unwrap();
        // event_thresholds 含 6 內建 + 飲食測試，順序保留。
        assert_eq!(settings.event_thresholds.len(), 7);
        assert_eq!(settings.event_thresholds[0].label, "空腹血糖");
        let dinner = settings
            .event_thresholds
            .iter()
            .find(|t| t.label == "晚餐後")
            .unwrap();
        assert_eq!((dinner.low, dinner.high), (70, 140));
        // custom_events：飲食測試查標準值表得 70/139。
        assert_eq!(settings.custom_events.len(), 1);
        assert_eq!(settings.custom_events[0].label, "飲食測試");
        assert_eq!(
            (
                settings.custom_events[0].low_threshold,
                settings.custom_events[0].high_threshold
            ),
            (70, 139)
        );
    }

    #[test]
    fn keyword_not_in_standards_uses_fallback() {
        // 關鍵字表含「運動後」，標準值表無此事件 → fallback 70/139。
        let keywords = "事件關鍵字\n運動後\n";
        let settings = parse_sheet_settings(keywords, STANDARDS_CSV).unwrap();
        let sport = settings
            .custom_events
            .iter()
            .find(|c| c.label == "運動後")
            .unwrap();
        assert_eq!((sport.low_threshold, sport.high_threshold), (70, 139));
    }

    #[test]
    fn builtin_keyword_collision_is_ignored() {
        // 關鍵字表含內建「空腹血糖」 → 略過，不產生 custom_events。
        let keywords = "事件關鍵字\n空腹血糖\n飲食測試\n";
        let settings = parse_sheet_settings(keywords, STANDARDS_CSV).unwrap();
        assert_eq!(settings.custom_events.len(), 1);
        assert_eq!(settings.custom_events[0].label, "飲食測試");
    }

    #[test]
    fn empty_keywords_sheet_yields_no_custom_events() {
        let settings = parse_sheet_settings("事件關鍵字\n", STANDARDS_CSV).unwrap();
        assert!(settings.custom_events.is_empty());
    }

    #[test]
    fn standards_missing_builtin_is_rejected() {
        // 缺「睡前」。
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("睡前")));
    }

    #[test]
    fn standards_wrong_header_is_rejected() {
        let bad = "事件,下限,上限\n空腹血糖,70,100\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("標頭")));
    }

    #[test]
    fn standards_empty_is_rejected() {
        let result = parse_sheet_settings(KEYWORDS_CSV, "事件,血糖下限,血糖上限\n");
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("無資料列")));
    }

    #[test]
    fn standards_non_numeric_is_rejected() {
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n睡前,abc,140\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("非有效整數")));
    }

    #[test]
    fn standards_out_of_range_is_rejected() {
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n睡前,19,140\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("20–600")));
    }

    #[test]
    fn standards_low_not_less_than_high_is_rejected() {
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,140,140\n睡前,70,140\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("下限須小於上限")));
    }

    #[test]
    fn standards_duplicate_label_is_rejected() {
        let bad = "事件,血糖下限,血糖上限\n空腹血糖,70,100\n空腹血糖,70,99\n午餐前,70,101\n午餐後,70,140\n晚餐前,70,101\n晚餐後,70,140\n睡前,70,140\n";
        let result = parse_sheet_settings(KEYWORDS_CSV, bad);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("重複出現")));
    }

    #[test]
    fn keywords_wrong_header_is_rejected() {
        let result = parse_sheet_settings("關鍵字\n飲食測試\n", STANDARDS_CSV);
        assert!(
            matches!(result, Err(AppError::Sync(msg)) if msg.contains("事件關鍵字設定") && msg.contains("標頭"))
        );
    }

    #[test]
    fn keywords_empty_label_is_rejected() {
        let result = parse_sheet_settings("事件關鍵字\n  \n", STANDARDS_CSV);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("不可為空")));
    }

    #[test]
    fn keywords_duplicate_is_rejected() {
        let result = parse_sheet_settings("事件關鍵字\n飲食測試\n飲食測試\n", STANDARDS_CSV);
        assert!(matches!(result, Err(AppError::Sync(msg)) if msg.contains("重複出現")));
    }

    #[test]
    fn builtin_fallback_has_six_builtins_no_custom() {
        let settings = builtin_fallback_settings();
        assert!(settings.custom_events.is_empty());
        assert_eq!(settings.event_thresholds.len(), 6);
    }

    #[test]
    fn standards_preserves_input_order() {
        // 自訂「飲食測試」在內建之後（列順序即 event_thresholds 順序）。
        let settings = parse_sheet_settings(KEYWORDS_CSV, STANDARDS_CSV).unwrap();
        let labels: Vec<&str> = settings
            .event_thresholds
            .iter()
            .map(|t| t.label.as_str())
            .collect();
        assert_eq!(
            labels,
            [
                "空腹血糖",
                "午餐前",
                "午餐後",
                "晚餐前",
                "晚餐後",
                "睡前",
                "飲食測試"
            ]
        );
    }
}
