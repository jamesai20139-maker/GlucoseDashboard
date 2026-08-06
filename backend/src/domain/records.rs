use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    Fasting,
    LunchBefore,
    LunchAfter,
    DinnerBefore,
    DinnerAfter,
    Bedtime,
}

impl Event {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "空腹血糖" => Some(Self::Fasting),
            "午餐前" => Some(Self::LunchBefore),
            "午餐後" => Some(Self::LunchAfter),
            "晚餐前" => Some(Self::DinnerBefore),
            "晚餐後" => Some(Self::DinnerAfter),
            "睡前" => Some(Self::Bedtime),
            _ => None,
        }
    }

    pub fn label_zh_tw(&self) -> &'static str {
        match self {
            Self::Fasting => "空腹血糖",
            Self::LunchBefore => "午餐前",
            Self::LunchAfter => "午餐後",
            Self::DinnerBefore => "晚餐前",
            Self::DinnerAfter => "晚餐後",
            Self::Bedtime => "睡前",
        }
    }

    pub fn is_before_meal(&self) -> bool {
        matches!(self, Self::LunchBefore | Self::DinnerBefore)
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Period {
    #[default]
    Day,
    Week,
    Month,
    Quarter,
    Custom {
        start: NaiveDate,
        end: NaiveDate,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSelection {
    pub period: Period,
    pub event: Option<Event>,
    pub search: Option<String>,
}

impl Period {
    pub fn contains(&self, value: DateTime<Utc>) -> bool {
        let date = value.date_naive();
        let today = Utc::now().date_naive();
        let start = match self {
            Self::Day => today,
            Self::Week => today - Duration::days(6),
            Self::Month => NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today),
            Self::Quarter => {
                let month = ((today.month0() / 3) * 3) + 1;
                NaiveDate::from_ymd_opt(today.year(), month, 1).unwrap_or(today)
            }
            Self::Custom { start, .. } => *start,
        };
        let end = match self {
            Self::Custom { end, .. } => *end,
            _ => today,
        };
        date >= start && date <= end
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
