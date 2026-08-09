use serde::{Deserialize, Serialize};

/// 使用者自訂事件關鍵字設定。重匯出 domain 的 `CustomEvent` 作為設定檔持久化形狀，
/// 避免重複定義；兩者欄位一致（label / low_threshold / high_threshold）。
pub use crate::domain::CustomEvent as CustomEventConfig;

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
}

impl LocalConfiguration {
    pub fn is_configured(&self) -> bool {
        self.sheet_id.is_some() || self.fixture_path.is_some()
    }
}
