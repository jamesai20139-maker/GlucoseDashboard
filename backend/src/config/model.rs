use serde::{Deserialize, Serialize};

/// 使用者自訂事件關鍵字。重匯出 domain 的 `CustomEvent` 作為執行期承載形狀，
/// 避免重複定義；兩者欄位一致（label / low_threshold / high_threshold）。
/// 注意：自 schema 4 起 `custom_events` 不再持久化於設定檔，改由 Google Sheet
/// 的「事件關鍵字設定」工作表即時衍生（見 `ingestion::settings_loader`）。
pub use crate::domain::CustomEvent as CustomEventConfig;

/// 單一事件的「前端顯示標準」範圍。`event_thresholds` 是趨勢圖與表格上色的
/// 唯一來源；後端摘要統計（`summary.rs`）不讀取此值，仍採內建醫學標準。
/// 自 schema 4 起不持久化，改由「血糖標準值設定」工作表即時衍生。
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
/// 用於本機 CSV（未連結 Sheet）時的退回預設，以及 settings_loader 驗證。
/// 回傳 `Vec` 而非 `const`，因 `EventThreshold.label` 為 `String` 無法在 const 上下文建構。
pub fn builtin_event_thresholds() -> Vec<EventThreshold> {
    vec![
        EventThreshold {
            label: "空腹血糖".into(),
            low: 70,
            high: 100,
        },
        EventThreshold {
            label: "午餐前".into(),
            low: 70,
            high: 101,
        },
        EventThreshold {
            label: "午餐後".into(),
            low: 70,
            high: 140,
        },
        EventThreshold {
            label: "晚餐前".into(),
            low: 70,
            high: 101,
        },
        EventThreshold {
            label: "晚餐後".into(),
            low: 70,
            high: 140,
        },
        EventThreshold {
            label: "睡前".into(),
            low: 70,
            high: 140,
        },
    ]
}

/// 目前設定檔 schema 版本。schema 4：移除持久化的 `custom_events`/
/// `event_thresholds`，改存「事件關鍵字設定」與「血糖標準值設定」兩個工作表
/// 名稱；該兩項設定改由 Google Sheet 工作表即時衍生，不再寫入本機 JSON。
pub const CURRENT_SCHEMA_VERSION: u32 = 4;

/// 「事件關鍵字設定」工作表的預設名稱。
pub const DEFAULT_EVENT_KEYWORDS_SHEET_NAME: &str = "事件關鍵字設定";
/// 「血糖標準值設定」工作表的預設名稱。
pub const DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME: &str = "血糖標準值設定";

/// 自訂事件後端固定 fallback 顯示標準（事件關鍵字未列於「血糖標準值設定」時用）。
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
    /// 「事件關鍵字設定」工作表名稱。舊設定檔（schema ≤3）無此欄位時回退 None，
    /// 由 `normalize()` 補上 `DEFAULT_EVENT_KEYWORDS_SHEET_NAME`。
    #[serde(default)]
    pub event_keywords_sheet_name: Option<String>,
    /// 「血糖標準值設定」工作表名稱。舊設定檔（schema ≤3）無此欄位時回退 None，
    /// 由 `normalize()` 補上 `DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME`。
    #[serde(default)]
    pub glucose_standards_sheet_name: Option<String>,
}

impl LocalConfiguration {
    pub fn is_configured(&self) -> bool {
        self.sheet_id.is_some() || self.fixture_path.is_some()
    }

    /// 正規化設定：補齊缺失的兩個工作表名稱（空白或 None → 預設常數）、
    /// 將 schema_version 升至目前版本。自 schema 4 起不再處理
    /// `custom_events`/`event_thresholds`（該兩項改由 Sheet 即時衍生，不持久化）。
    /// 舊設定檔中殘留的 `custom_events`/`event_thresholds` 欄位會被 Serde 忽略，
    /// 下次 save 即消失（依憲法不保留 Sheet 衍生資料）。
    pub fn normalize(&mut self) {
        if self
            .event_keywords_sheet_name
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            self.event_keywords_sheet_name = Some(DEFAULT_EVENT_KEYWORDS_SHEET_NAME.to_string());
        } else {
            self.event_keywords_sheet_name = Some(
                self.event_keywords_sheet_name
                    .clone()
                    .unwrap()
                    .trim()
                    .to_string(),
            );
        }
        if self
            .glucose_standards_sheet_name
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            self.glucose_standards_sheet_name =
                Some(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME.to_string());
        } else {
            self.glucose_standards_sheet_name = Some(
                self.glucose_standards_sheet_name
                    .clone()
                    .unwrap()
                    .trim()
                    .to_string(),
            );
        }
        if self.schema_version < CURRENT_SCHEMA_VERSION {
            self.schema_version = CURRENT_SCHEMA_VERSION;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> LocalConfiguration {
        LocalConfiguration {
            schema_version: 0,
            sheet_id: None,
            sheet_gid: None,
            sheet_name: None,
            fixture_path: None,
            credential_reference: None,
            last_successful_sync_at: None,
            event_keywords_sheet_name: None,
            glucose_standards_sheet_name: None,
        }
    }

    #[test]
    fn builtin_thresholds_match_classify_rules() {
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
    fn normalize_fills_default_worksheet_names() {
        let mut config = empty_config();
        config.normalize();
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some(DEFAULT_EVENT_KEYWORDS_SHEET_NAME)
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME)
        );
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn normalize_preserves_explicit_worksheet_names_and_trims() {
        let mut config = empty_config();
        config.event_keywords_sheet_name = Some("  我的關鍵字  ".into());
        config.glucose_standards_sheet_name = Some("我的標準值".into());
        config.normalize();
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some("我的關鍵字")
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some("我的標準值")
        );
    }

    #[test]
    fn normalize_treats_blank_names_as_missing() {
        let mut config = empty_config();
        config.event_keywords_sheet_name = Some("   ".into());
        config.glucose_standards_sheet_name = Some("".into());
        config.normalize();
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some(DEFAULT_EVENT_KEYWORDS_SHEET_NAME)
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME)
        );
    }

    #[test]
    fn normalize_does_not_downgrade_schema_version() {
        let mut config = empty_config();
        config.schema_version = CURRENT_SCHEMA_VERSION;
        config.normalize();
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn normalize_upgrades_old_schema_to_4() {
        let mut config = empty_config();
        config.schema_version = 3;
        config.normalize();
        assert_eq!(config.schema_version, 4);
    }

    #[test]
    fn old_json_with_legacy_vectors_is_ignored() {
        // 模擬 schema 3 舊設定檔仍含 custom_events/event_thresholds（應被忽略）。
        let json = r#"{
            "schema_version": 3,
            "sheet_id": "ABC",
            "sheet_gid": null,
            "sheet_name": "Sheet1",
            "fixture_path": null,
            "credential_reference": null,
            "last_successful_sync_at": null,
            "custom_events": [{"label":"運動後","low_threshold":80,"high_threshold":120}],
            "event_thresholds": [{"label":"空腹血糖","low":70,"high":99}]
        }"#;
        let mut config: LocalConfiguration = serde_json::from_str(json).unwrap();
        config.normalize();
        // 工作表名補預設、schema 升 4。
        assert_eq!(config.schema_version, 4);
        assert_eq!(
            config.event_keywords_sheet_name.as_deref(),
            Some(DEFAULT_EVENT_KEYWORDS_SHEET_NAME)
        );
        assert_eq!(
            config.glucose_standards_sheet_name.as_deref(),
            Some(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME)
        );
        // 舊欄位不持久化：序列化後不應出現 custom_events/event_thresholds。
        let persisted = serde_json::to_string(&config).unwrap();
        assert!(!persisted.contains("custom_events"));
        assert!(!persisted.contains("event_thresholds"));
    }

    #[test]
    fn event_threshold_serializes_with_expected_fields() {
        let t = EventThreshold {
            label: "空腹血糖".into(),
            low: 70,
            high: 99,
        };
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"label":"空腹血糖","low":70,"high":99}"#);
    }
}
