use serde::{Deserialize, Serialize};

/// 使用者自訂事件關鍵字設定。重匯出 domain 的 `CustomEvent` 作為設定檔持久化形狀，
/// 避免重複定義；兩者欄位一致（label / low_threshold / high_threshold）。
pub use crate::domain::CustomEvent as CustomEventConfig;

/// 單一事件的「前端顯示標準」範圍。`event_thresholds` 是趨勢圖與表格上色的
/// 唯一來源；後端摘要統計（`summary.rs`）不讀取此值，仍採內建醫學標準。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventThreshold {
    pub label: String,
    pub low: i32,
    pub high: i32,
}

/// 內建事件名稱（固定順序：空腹/午餐前/午餐後/晚餐前/晚餐後/睡前）。
pub const BUILTIN_EVENT_LABELS: [&str; 6] =
    ["空腹血糖", "午餐前", "午餐後", "晚餐前", "晚餐後", "睡前"];

/// 6 個內建事件的預設顯示標準範圍。前端以 `value >= high` 判高，故 `high` 採用
/// 「第一個偏高值」以符合臨床慣例：餐後/睡前 `high = 140`（140 判高）、空腹
/// `high = 100`、餐前 `high = 101`。後端摘要統計（`summary.rs`）仍用
/// `GlucoseRecord::classify()` 的內建醫學標準，不讀取此值。
/// 回傳 `Vec` 而非 `const`，因 `EventThreshold.label` 為 `String` 無法在 const 上下文建構。
pub fn builtin_event_thresholds() -> Vec<EventThreshold> {
    vec![
        EventThreshold { label: "空腹血糖".into(), low: 70, high: 100 },
        EventThreshold { label: "午餐前".into(), low: 70, high: 101 },
        EventThreshold { label: "午餐後".into(), low: 70, high: 140 },
        EventThreshold { label: "晚餐前".into(), low: 70, high: 101 },
        EventThreshold { label: "晚餐後".into(), low: 70, high: 140 },
        EventThreshold { label: "睡前".into(), low: 70, high: 140 },
    ]
}

/// 目前設定檔 schema 版本。新增 `event_thresholds` 欄位時由 2 升至 3。
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

/// 自訂事件後端固定 fallback 顯示標準（不影響摘要，僅供解析預設）。
pub const CUSTOM_EVENT_FALLBACK_LOW: i32 = 70;
pub const CUSTOM_EVENT_FALLBACK_HIGH: i32 = 139;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalConfiguration {
    pub schema_version: u32,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<String>,
    pub credential_reference: Option<String>,
    pub last_successful_sync_at: Option<String>,
    /// 使用者自訂事件關鍵字。舊設定檔（v1）無此欄位時回退為空 list。
    #[serde(default)]
    pub custom_events: Vec<CustomEventConfig>,
    /// 每個事件的前端顯示標準範圍。舊設定檔（v2）無此欄位時回退為空 list，
    /// 由 `normalize()` 補齊內建預設值。
    #[serde(default)]
    pub event_thresholds: Vec<EventThreshold>,
}

impl LocalConfiguration {
    pub fn is_configured(&self) -> bool {
        self.sheet_id.is_some() || self.fixture_path.is_some()
    }

    /// 正規化設定：補齊缺失的內建閾值、去重、固定順序（6 內建在前，自訂在後），
    /// 並將 schema_version 升至目前版本。用於舊設定檔 migration（schema 2→3）與
    /// 新設定寫入前的整備。
    pub fn normalize(&mut self) {
        // 若 event_thresholds 缺少任一內建事件，補上預設值。
        for builtin in builtin_event_thresholds() {
            if !self.event_thresholds.iter().any(|t| t.label == builtin.label) {
                self.event_thresholds.push(builtin);
            }
        }
        // Migration：舊設定（schema 2）自訂事件閾值存於 custom_events，搬到
        // event_thresholds 作為前端顯示標準來源，避免使用者原設標準遺失。
        // 同 label 已存在於 event_thresholds 則以現有為準（不覆蓋使用者新設定）。
        for c in &self.custom_events {
            if !self.event_thresholds.iter().any(|t| t.label == c.label) {
                self.event_thresholds.push(EventThreshold {
                    label: c.label.clone(),
                    low: c.low_threshold,
                    high: c.high_threshold,
                });
            }
        }
        // 去重：保留每個 label 第一次出現的項目。
        let mut seen: Vec<String> = Vec::new();
        self.event_thresholds.retain(|t| {
            if seen.iter().any(|s| s == &t.label) {
                false
            } else {
                seen.push(t.label.clone());
                true
            }
        });
        // 固定順序：6 內建在前（依 BUILTIN_EVENT_LABELS 順序），其餘自訂在後（依現有順序）。
        self.event_thresholds.sort_by_key(|t| {
            BUILTIN_EVENT_LABELS
                .iter()
                .position(|label| label == &t.label)
                .map(|i| (0, i))
                .unwrap_or((1, 0))
        });
        if self.schema_version < CURRENT_SCHEMA_VERSION {
            self.schema_version = CURRENT_SCHEMA_VERSION;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn threshold(label: &str, low: i32, high: i32) -> EventThreshold {
        EventThreshold { label: label.into(), low, high }
    }

    #[test]
    fn builtin_thresholds_match_classify_rules() {
        // 餐後/睡前 high=139：140 應被前端判為高（value > high）。
        let builtins = builtin_event_thresholds();
        assert_eq!(builtins.len(), 6);
        for t in &builtins {
            assert!((20..=600).contains(&t.low));
            assert!((20..=600).contains(&t.high));
            assert!(t.low < t.high);
        }
        let dinner_after = builtins.iter().find(|t| t.label == "晚餐後").unwrap();
        assert_eq!((dinner_after.low, dinner_after.high), (70, 140));
    }

    #[test]
    fn normalize_adds_missing_builtin_thresholds() {
        let mut config = LocalConfiguration {
            schema_version: 2,
            sheet_id: Some("ABC".into()),
            sheet_gid: None,
            sheet_name: Some("Sheet1".into()),
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            event_thresholds: Vec::new(),
        };
        config.normalize();
        // 6 內建全部補齊。
        assert_eq!(config.event_thresholds.len(), 6);
        assert_eq!(config.event_thresholds[0].label, "空腹血糖");
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn normalize_preserves_user_overrides_and_order() {
        let mut config = LocalConfiguration {
            schema_version: 2,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            event_thresholds: vec![
                // 使用者把空腹改成 70/110；順序刻意打亂。
                threshold("空腹血糖", 70, 110),
                // 自訂事件在前。
                threshold("運動後", 80, 120),
            ],
        };
        config.normalize();
        // 內建在前、自訂在後；使用者覆蓋值保留。
        assert_eq!(config.event_thresholds[0], threshold("空腹血糖", 70, 110));
        assert_eq!(
            config.event_thresholds.iter().find(|t| t.label == "午餐前"),
            Some(&threshold("午餐前", 70, 101))
        );
        assert_eq!(
            config.event_thresholds.last(),
            Some(&threshold("運動後", 80, 120))
        );
    }

    #[test]
    fn normalize_deduplicates_by_label() {
        let mut config = LocalConfiguration {
            schema_version: 3,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            event_thresholds: vec![
                threshold("空腹血糖", 70, 110),
                threshold("空腹血糖", 70, 99), // 重複，應被丟棄
            ],
        };
        config.normalize();
        let fasting = config
            .event_thresholds
            .iter()
            .filter(|t| t.label == "空腹血糖")
            .count();
        assert_eq!(fasting, 1);
        // 保留第一次出現的值。
        assert_eq!(
            config.event_thresholds.iter().find(|t| t.label == "空腹血糖"),
            Some(&threshold("空腹血糖", 70, 110))
        );
    }

    #[test]
    fn normalize_does_not_downgrade_schema_version() {
        let mut config = LocalConfiguration {
            schema_version: CURRENT_SCHEMA_VERSION,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            event_thresholds: Vec::new(),
        };
        config.normalize();
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn normalize_sorts_builtins_before_custom() {
        let mut config = LocalConfiguration {
            schema_version: 3,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: Vec::new(),
            // 自訂在前、內建散落其後。
            event_thresholds: vec![
                threshold("運動後", 70, 139),
                threshold("睡前", 70, 139),
                threshold("空腹血糖", 70, 99),
            ],
        };
        config.normalize();
        let labels: Vec<&str> =
            config.event_thresholds.iter().map(|t| t.label.as_str()).collect();
        // 內建依 BUILTIN_EVENT_LABELS 順序在前，自訂在後。
        assert_eq!(labels[0], "空腹血糖");
        assert_eq!(labels[1], "午餐前");
        assert_eq!(labels[2], "午餐後");
        assert_eq!(labels[3], "晚餐前");
        assert_eq!(labels[4], "晚餐後");
        assert_eq!(labels[5], "睡前");
        assert_eq!(labels[6], "運動後");
    }

    #[test]
    fn normalize_migrates_custom_event_thresholds() {
        // 舊設定（schema 2）：自訂事件閾值存於 custom_events，event_thresholds 為空。
        let mut config = LocalConfiguration {
            schema_version: 2,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: vec![CustomEventConfig {
                label: "運動後".into(),
                low_threshold: 80,
                high_threshold: 120,
            }],
            event_thresholds: Vec::new(),
        };
        config.normalize();
        // 自訂事件閾值應搬到 event_thresholds，使用者原設標準保留。
        let sport = config
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (80, 120));
        // 6 內建亦補齊。
        assert_eq!(config.event_thresholds.len(), 7);
    }

    #[test]
    fn normalize_does_not_overwrite_existing_threshold_from_custom_events() {
        // event_thresholds 已有「運動後」新設值，custom_events 的舊閾值不應覆蓋。
        let mut config = LocalConfiguration {
            schema_version: 3,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            custom_events: vec![CustomEventConfig {
                label: "運動後".into(),
                low_threshold: 80,
                high_threshold: 120,
            }],
            event_thresholds: vec![threshold("運動後", 70, 139)],
        };
        config.normalize();
        let sport = config
            .event_thresholds
            .iter()
            .find(|t| t.label == "運動後")
            .unwrap();
        assert_eq!((sport.low, sport.high), (70, 139));
    }

    #[test]
    fn event_threshold_serializes_with_expected_fields() {
        let t = threshold("空腹血糖", 70, 99);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(
            json,
            r#"{"label":"空腹血糖","low":70,"high":99}"#
        );
    }
}