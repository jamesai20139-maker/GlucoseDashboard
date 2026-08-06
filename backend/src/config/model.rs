use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalConfiguration {
    pub schema_version: u32,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<String>,
    pub credential_reference: Option<String>,
    pub last_successful_sync_at: Option<String>,
}

impl LocalConfiguration {
    pub fn is_configured(&self) -> bool {
        self.sheet_id.is_some() || self.fixture_path.is_some()
    }
}
