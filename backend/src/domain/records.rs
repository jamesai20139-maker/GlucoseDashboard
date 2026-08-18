use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::data_quality::DataQualityIssue;

/// 表格顯示用列：包含 Sheet 每一列（含格式錯誤列）。顯示值為 `Option<String>`，
/// `None` 表示該欄位格式錯誤（前端顯示 "Type Error"）。`parsed_*` 供後端篩選，
/// 不序列化送出前端。
#[derive(Debug, Clone, Serialize)]
pub struct DashboardTableRow {
    pub source_row_number: usize,
    pub measured_at: Option<String>,
    pub event: Option<String>,
    pub glucose_mg_dl: Option<String>,
    pub remark_1: String,
    pub remark_2: String,
    #[serde(skip)]
    pub parsed_measured_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub parsed_event: Option<Event>,
}

/// 自訂事件關鍵字：由使用者在設定介面新增，攜帶自訂高低閾值。同時作為設定檔
/// 持久化形狀與 `Event::Custom` 的執行期承載，避免額外的 DTO 轉換。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomEvent {
    pub label: String,
    /// 低於此值視為低血糖。
    pub low_threshold: i32,
    /// 高於此值視為高血糖。
    pub high_threshold: i32,
}

/// 事件種類。前 6 個為內建事件（固定中文名稱），`Custom` 為使用者自訂關鍵字。
/// 序列化時一律以顯示標籤字串（內建為中文、自訂為其 label）呈現，讓前端可統一以
/// 字串處理；`PartialEq` 僅以標籤比較，確保篩選重建的事件與紀錄內事件相符。
#[derive(Debug, Clone)]
pub enum Event {
    Fasting,
    LunchBefore,
    LunchAfter,
    DinnerBefore,
    DinnerAfter,
    Bedtime,
    Custom(CustomEvent),
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.label_zh_tw() == other.label_zh_tw()
    }
}

impl Eq for Event {}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.label_zh_tw())
    }
}

impl<'de> Deserialize<'de> for Event {
    /// 反序列化僅出現在 `AnalysisSelection` 的 derive 路徑，目前無 wire 端點
    /// 會把自訂事件標籤反序列化進來；此處對未知標籤回退為一個 `Custom` 事件
    /// （閾值 20–600，即「全段正常」），因其只會用於依標籤篩選、不參與分類。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::parse(&label, &[]).unwrap_or_else(|| {
            Self::Custom(CustomEvent {
                label,
                low_threshold: 20,
                high_threshold: 600,
            })
        }))
    }
}

impl Event {
    pub fn parse(value: &str, custom: &[CustomEvent]) -> Option<Self> {
        match value.trim() {
            "空腹血糖" => Some(Self::Fasting),
            "午餐前" => Some(Self::LunchBefore),
            "午餐後" => Some(Self::LunchAfter),
            "晚餐前" => Some(Self::DinnerBefore),
            "晚餐後" => Some(Self::DinnerAfter),
            "睡前" => Some(Self::Bedtime),
            _ => custom
                .iter()
                .find(|c| c.label == value.trim())
                .map(|c| Self::Custom(c.clone())),
        }
    }

    pub fn label_zh_tw(&self) -> String {
        match self {
            Self::Fasting => "空腹血糖".into(),
            Self::LunchBefore => "午餐前".into(),
            Self::LunchAfter => "午餐後".into(),
            Self::DinnerBefore => "晚餐前".into(),
            Self::DinnerAfter => "晚餐後".into(),
            Self::Bedtime => "睡前".into(),
            Self::Custom(c) => c.label.clone(),
        }
    }

    pub fn is_before_meal(&self) -> bool {
        matches!(self, Self::LunchBefore | Self::DinnerBefore)
    }

    /// 是否為餐後/睡前事件。`Custom` 事件不屬於任何餐時情境，回 `false`。
    /// 保留為公共 API；目前未被分類邏輯直接使用（餐後分支以閾值兜底）。
    #[allow(dead_code)]
    pub fn is_post_meal(&self) -> bool {
        matches!(self, Self::LunchAfter | Self::DinnerAfter | Self::Bedtime)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Classification {
    Low,
    InRange,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlucoseRecord {
    pub source_row_number: usize,
    pub measured_at: DateTime<Utc>,
    pub event: Event,
    pub glucose_mg_dl: i32,
    pub remark_1: String,
    pub remark_2: String,
}

impl GlucoseRecord {
    pub fn classify(&self) -> Classification {
        let value = self.glucose_mg_dl;
        match &self.event {
            // 自訂事件：`high` 採「第一個偏高值」語義（value >= high 判高），
            // 與前端 `classify.ts` 的 `value >= high` 及內建事件閾值語義一致。
            Event::Custom(c) => {
                if value < c.low_threshold {
                    Classification::Low
                } else if value >= c.high_threshold {
                    Classification::High
                } else {
                    Classification::InRange
                }
            }
            Event::Fasting
            | Event::LunchBefore
            | Event::LunchAfter
            | Event::DinnerBefore
            | Event::DinnerAfter
            | Event::Bedtime => {
                if value < 70 {
                    Classification::Low
                } else if self.event == Event::Fasting {
                    if value <= 99 {
                        Classification::InRange
                    } else {
                        Classification::High
                    }
                } else if self.event.is_before_meal() {
                    if value <= 100 {
                        Classification::InRange
                    } else {
                        Classification::High
                    }
                } else if value < 140 {
                    Classification::InRange
                } else {
                    Classification::High
                }
            }
        }
    }
}

/// 時間區間。變體皆為「明確區間」（非相對今天），讓使用者可指定任意
/// 年/週/月/季/日起訖，由前端傳入細粒 query 參數組裝。
///
/// 週採「台灣常用」定義：以該年 1/1 為第 1 週第 1 天，第 N 週 =
/// 1/1 + (N-1)*7 天起的連續 7 天（可能跨年，含示）。季採 Q1=1-3 月、
/// Q2=4-6 月、Q3=7-9 月、Q4=10-12 月。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Period {
    /// 全部資料，`contains()` 恆真。
    All,
    /// 自訂起訖日期區間（含兩端）。
    Day { start: NaiveDate, end: NaiveDate },
    /// 某年第 N 週（1/1 起算）。
    Week { year: i32, week: u32 },
    /// 某年某月。
    Month { year: i32, month: u32 },
    /// 某年某季（1..=4）。
    Quarter { year: i32, quarter: u32 },
}

impl Default for Period {
    /// 預設顯示全部資料。
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSelection {
    pub period: Period,
    pub event: Option<Event>,
    pub search: Option<String>,
}

impl Period {
    /// 將此區間對應的 `[start, end]`（含兩端）`NaiveDate` 算出。
    /// `All` 回傳 `None`（`contains()` 對所有日期恆真，不需邊界）。
    pub fn date_range(&self) -> Option<(NaiveDate, NaiveDate)> {
        match self {
            Self::All => None,
            Self::Day { start, end } => Some((*start, *end)),
            Self::Week { year, week } => {
                let year_start = NaiveDate::from_ymd_opt(*year, 1, 1)?;
                // 第 1 週從 1/1 開始；第 N 週從 1/1 + (N-1)*7 天開始。
                let start = year_start + Duration::days(((*week as i64) - 1) * 7);
                let end = start + Duration::days(6);
                Some((start, end))
            }
            Self::Month { year, month } => {
                let start = NaiveDate::from_ymd_opt(*year, *month, 1)?;
                // 月底：下月 1 號減 1 天（12 月跨年時取次年 1/1 減 1）。
                let next_month = if *month == 12 {
                    NaiveDate::from_ymd_opt(*year + 1, 1, 1)?
                } else {
                    NaiveDate::from_ymd_opt(*year, *month + 1, 1)?
                };
                let end = next_month - Duration::days(1);
                Some((start, end))
            }
            Self::Quarter { year, quarter } => {
                // Q1=1-3 月、Q2=4-6 月、Q3=7-9 月、Q4=10-12 月。
                let start_month = ((*quarter - 1) * 3) + 1;
                let end_month = start_month + 2;
                let start = NaiveDate::from_ymd_opt(*year, start_month, 1)?;
                let next_after_end = if end_month == 12 {
                    NaiveDate::from_ymd_opt(*year + 1, 1, 1)?
                } else {
                    NaiveDate::from_ymd_opt(*year, end_month + 1, 1)?
                };
                let end = next_after_end - Duration::days(1);
                Some((start, end))
            }
        }
    }

    /// 此 `DateTime` 的日期是否落在區間內。`All` 恆真。
    pub fn contains(&self, value: DateTime<Utc>) -> bool {
        let date = value.date_naive();
        match self.date_range() {
            None => true,
            Some((start, end)) => date >= start && date <= end,
        }
    }

    /// 繁體中文顯示標籤，供前端顯示當前區間。
    pub fn label_zh_tw(&self) -> String {
        match self {
            Self::All => "全部".to_string(),
            Self::Day { start, end } => format!("{}～{}", start, end),
            Self::Week { year, week } => format!("{} 年第 {} 週", year, week),
            Self::Month { year, month } => format!("{} 年 {} 月", year, month),
            Self::Quarter { year, quarter } => format!("{} 年第 {} 季", year, quarter),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisSummary {
    pub record_count: usize,
    pub average_mg_dl: Option<f64>,
    pub minimum_mg_dl: Option<i32>,
    pub maximum_mg_dl: Option<i32>,
    pub estimated_hba1c_percent: Option<f64>,
    pub estimated_average_glucose_mg_dl: Option<f64>,
    pub in_reference_percent: Option<f64>,
    pub low_percent: Option<f64>,
    pub high_percent: Option<f64>,
}

#[allow(dead_code)]
pub type ParseResult = (Vec<GlucoseRecord>, Vec<DataQualityIssue>);

#[cfg(test)]
mod tests {
    use super::{Classification, CustomEvent, Event, GlucoseRecord, Period};
    use chrono::{DateTime, NaiveDate, Utc};

    fn dt(date: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date))
            .unwrap()
            .with_timezone(&Utc)
    }

    fn day(start: &str, end: &str) -> Period {
        Period::Day {
            start: NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap(),
            end: NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap(),
        }
    }

    #[test]
    fn all_contains_every_date() {
        let period = Period::All;
        assert!(period.contains(dt("2000-01-01")));
        assert!(period.contains(dt("2099-12-31")));
        assert_eq!(period.label_zh_tw(), "全部");
    }

    #[test]
    fn day_range_is_inclusive() {
        let period = day("2026-03-01", "2026-03-31");
        assert!(period.contains(dt("2026-03-01")));
        assert!(period.contains(dt("2026-03-31")));
        assert!(!period.contains(dt("2026-02-28")));
        assert!(!period.contains(dt("2026-04-01")));
    }

    #[test]
    fn week_starts_on_jan_1() {
        // 2026/1/1 為第 1 週第 1 天 → 1/1..1/7。
        let period = Period::Week {
            year: 2026,
            week: 1,
        };
        assert!(period.contains(dt("2026-01-01")));
        assert!(period.contains(dt("2026-01-07")));
        assert!(!period.contains(dt("2025-12-31")));
        assert!(!period.contains(dt("2026-01-08")));
        // 第 2 週 = 1/8..1/14。
        let period2 = Period::Week {
            year: 2026,
            week: 2,
        };
        assert!(period2.contains(dt("2026-01-08")));
        assert!(period2.contains(dt("2026-01-14")));
        assert!(
            period2.date_range()
                == Some((
                    NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
                    NaiveDate::from_ymd_opt(2026, 1, 14).unwrap()
                ))
        );
    }

    #[test]
    fn month_range_covers_full_month() {
        let period = Period::Month {
            year: 2026,
            month: 2,
        };
        assert!(period.contains(dt("2026-02-01")));
        assert!(period.contains(dt("2026-02-28")));
        assert!(!period.contains(dt("2026-01-31")));
        assert!(!period.contains(dt("2026-03-01")));
        // 閏年二月有 29 天。
        let leap = Period::Month {
            year: 2024,
            month: 2,
        };
        assert!(leap.contains(dt("2024-02-29")));
    }

    #[test]
    fn december_month_range_does_not_overflow() {
        let period = Period::Month {
            year: 2026,
            month: 12,
        };
        assert!(period.contains(dt("2026-12-01")));
        assert!(period.contains(dt("2026-12-31")));
        assert!(!period.contains(dt("2027-01-01")));
    }

    #[test]
    fn quarter_ranges() {
        // Q1 = 1-3 月。
        let q1 = Period::Quarter {
            year: 2026,
            quarter: 1,
        };
        assert!(q1.contains(dt("2026-01-01")));
        assert!(q1.contains(dt("2026-03-31")));
        assert!(!q1.contains(dt("2025-12-31")));
        assert!(!q1.contains(dt("2026-04-01")));
        // Q4 = 10-12 月，跨年邊界。
        let q4 = Period::Quarter {
            year: 2026,
            quarter: 4,
        };
        assert!(q4.contains(dt("2026-10-01")));
        assert!(q4.contains(dt("2026-12-31")));
        assert!(!q4.contains(dt("2027-01-01")));
        assert!(!q4.contains(dt("2026-09-30")));
    }

    #[test]
    fn labels_are_traditional_chinese() {
        assert_eq!(
            Period::Week {
                year: 2026,
                week: 5
            }
            .label_zh_tw(),
            "2026 年第 5 週"
        );
        assert_eq!(
            Period::Month {
                year: 2026,
                month: 7
            }
            .label_zh_tw(),
            "2026 年 7 月"
        );
        assert_eq!(
            Period::Quarter {
                year: 2026,
                quarter: 2
            }
            .label_zh_tw(),
            "2026 年第 2 季"
        );
        assert_eq!(
            day("2026-03-01", "2026-03-31").label_zh_tw(),
            "2026-03-01～2026-03-31"
        );
    }

    #[test]
    fn default_is_all() {
        assert_eq!(Period::default(), Period::All);
    }

    fn custom(label: &str, low: i32, high: i32) -> CustomEvent {
        CustomEvent {
            label: label.into(),
            low_threshold: low,
            high_threshold: high,
        }
    }

    fn record(value: i32, event: Event) -> GlucoseRecord {
        GlucoseRecord {
            source_row_number: 1,
            measured_at: DateTime::parse_from_rfc3339("2026-07-07T06:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            event,
            glucose_mg_dl: value,
            remark_1: String::new(),
            remark_2: String::new(),
        }
    }

    #[test]
    fn parse_recognizes_builtin_and_custom_labels() {
        assert_eq!(Event::parse("空腹血糖", &[]), Some(Event::Fasting));
        assert_eq!(Event::parse("運動後", &[]), None);
        let defs = [custom("運動後", 70, 139)];
        assert_eq!(
            Event::parse("運動後", &defs),
            Some(Event::Custom(custom("運動後", 70, 139)))
        );
        // 自訂標籤前後空白會被 trim。
        assert_eq!(
            Event::parse("  運動後  ", &defs),
            Some(Event::Custom(custom("運動後", 70, 139)))
        );
        // 內建標籤不受自訂清單影響。
        assert_eq!(Event::parse("午餐前", &defs), Some(Event::LunchBefore));
    }

    #[test]
    fn custom_event_classifies_by_user_thresholds() {
        // `high` 採「第一個偏高值」語義（value >= high 判高），與前端及內建一致。
        let event = Event::Custom(custom("運動後", 70, 139));
        assert_eq!(record(69, event.clone()).classify(), Classification::Low);
        assert_eq!(
            record(70, event.clone()).classify(),
            Classification::InRange
        );
        assert_eq!(
            record(138, event.clone()).classify(),
            Classification::InRange
        );
        // 139 等於 high → 偏高。
        assert_eq!(record(139, event.clone()).classify(), Classification::High);
        assert_eq!(record(140, event).classify(), Classification::High);
    }

    #[test]
    fn event_equality_is_by_label() {
        // 即使閾值不同，只要標籤相同即視為相等，讓篩選重建的事件能與紀錄相符。
        assert_eq!(
            Event::Custom(custom("運動後", 70, 139)),
            Event::Custom(custom("運動後", 80, 120))
        );
        assert_ne!(Event::Custom(custom("運動後", 70, 139)), Event::Fasting);
    }

    #[test]
    fn custom_label_serializes_as_label_string() {
        let event = Event::Custom(custom("運動後", 70, 139));
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"運動後\"");
    }

    #[test]
    fn builtin_label_serializes_as_chinese_string() {
        let json = serde_json::to_string(&Event::Fasting).unwrap();
        assert_eq!(json, "\"空腹血糖\"");
    }
}
