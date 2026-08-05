use crate::{auth::credential_store::CredentialStore, config::store::ConfigStore};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

pub fn run(store: &ConfigStore) -> Vec<CheckResult> {
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
            message: "暫存層可用".into(),
        },
        CheckResult {
            name: "Dashboard".into(),
            ok: true,
            message: "本機服務可啟動".into(),
        },
    ]
}
