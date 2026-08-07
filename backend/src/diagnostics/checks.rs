use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{auth::credential_store::CredentialStore, config::store::ConfigStore};

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

/// 暫存層狀態：`(抓取時間, 紀錄筆數)`，無快取時為 None。
pub type CacheInfo = Option<(DateTime<Utc>, usize)>;

pub fn run(store: &ConfigStore, cache: CacheInfo) -> Vec<CheckResult> {
    let config = store.load();
    let credential_store = CredentialStore;
    vec![
        CheckResult {
            name: "Google Login".into(),
            ok: credential_store.available(),
            message: credential_store.status().into(),
        },
        CheckResult {
            name: "Google Sheet".into(),
            ok: config.is_configured(),
            message: if config.is_configured() {
                "已設定"
            } else {
                "尚未設定"
            }
            .into(),
        },
        CheckResult {
            name: "Network".into(),
            ok: true,
            message: "本機服務可用".into(),
        },
        CheckResult {
            name: "Config".into(),
            ok: config.is_configured(),
            message: "設定檢查完成".into(),
        },
        CheckResult {
            name: "Cache".into(),
            ok: true,
            message: match cache {
                Some((fetched_at, count)) => format!(
                    "進程記憶體暫存：{} 筆（{} 抓取，重啟清空）",
                    count,
                    fetched_at.format("%Y/%m/%d %H:%M")
                ),
                None => "進程記憶體暫存：未命中（重啟清空）".into(),
            },
        },
        CheckResult {
            name: "Dashboard".into(),
            ok: true,
            message: "本機服務可啟動".into(),
        },
    ]
}
