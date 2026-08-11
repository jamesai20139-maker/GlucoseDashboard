use crate::config::model::LocalConfiguration;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl Default for ConfigStore {
    fn default() -> Self {
        let path = default_config_path();
        Self { path }
    }
}

/// 決定預設設定檔路徑，順序如下：
/// 1. `GLUCOSE_CONFIG_PATH` 環境變數（明確覆寫，測試與自訂安裝位置用）。
/// 2. 當前工作目錄的 `.glucose-dashboard.json`（開發時 `make run` / `cargo run`
///    從 repo 根目錄啟動，沿用既有設定檔，零破壞）。
/// 3. 可執行檔同目錄的 `.glucose-dashboard.json`（安裝版：使用者從 PATH shim
///    或開始選單啟動時 CWD 不確定，設定檔應落在 exe 旁，避免亂跑或寫入
///    `C:\Windows` 等不可寫目錄）。
fn default_config_path() -> PathBuf {
    let env_path = std::env::var_os("GLUCOSE_CONFIG_PATH").map(PathBuf::from);
    let cwd = std::env::current_dir().ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    resolve_config_path(env_path, cwd, exe_dir)
}

/// 純邏輯解析設定檔路徑（不碰全域狀態，供單元測試注入）。
fn resolve_config_path(
    env_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
    exe_dir: Option<PathBuf>,
) -> PathBuf {
    if let Some(p) = env_path {
        return p;
    }
    let name = PathBuf::from(".glucose-dashboard.json");
    if let Some(cwd) = cwd {
        let candidate = cwd.join(&name);
        if candidate.is_file() {
            return candidate;
        }
    }
    if let Some(dir) = exe_dir {
        return dir.join(name);
    }
    // 最後回退：當前目錄相對路徑（與歷史行為一致）。
    name
}

impl ConfigStore {
    /// 以指定路徑建構（主要供測試注入，避免並行測試經環境變數競爭同一設定檔）。
    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> LocalConfiguration {
        let mut config: LocalConfiguration = fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        // 舊設定檔 migration：補齊內建閾值、去重、排序、升 schema 版本。
        config.normalize();
        config
    }

    pub fn save(&self, config: &LocalConfiguration) -> Result<(), String> {
        let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
        fs::write(&self.path, text).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_config_path;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 建立一個「每次呼叫唯一」的暫存目錄並在裡面放空的
    /// .glucose-dashboard.json，回傳該目錄。原子計數器確保並行測試不共用目錄。
    fn temp_cwd_with_config() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("glucose-config-test-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".glucose-dashboard.json"), b"{}").unwrap();
        dir
    }

    #[test]
    fn env_var_takes_priority_over_cwd_file() {
        // 即使 CWD 有 .glucose-dashboard.json，明確的 env 變數優先。
        let cwd = temp_cwd_with_config();
        let exe = PathBuf::from("/opt/app");
        let path = resolve_config_path(
            Some(PathBuf::from("/tmp/explicit-config.json")),
            Some(cwd.clone()),
            Some(exe),
        );
        assert_eq!(path, PathBuf::from("/tmp/explicit-config.json"));
        std::fs::remove_dir_all(&cwd).ok();
    }

    #[test]
    fn cwd_file_preferred_over_exe_dir() {
        // CWD 有 config 檔時，優先於 exe 同目錄。
        let cwd = temp_cwd_with_config();
        let exe = PathBuf::from("/opt/app");
        let path = resolve_config_path(None, Some(cwd.clone()), Some(exe));
        assert_eq!(path, cwd.join(".glucose-dashboard.json"));
        std::fs::remove_dir_all(&cwd).ok();
    }

    #[test]
    fn exe_dir_used_when_cwd_has_no_config_file() {
        // CWD 無 config 檔時，回退到 exe 同目錄（安裝版情境）。
        let cwd = PathBuf::from("/some/cwd/without/config");
        let exe = PathBuf::from("/opt/app");
        let path = resolve_config_path(None, Some(cwd), Some(exe));
        assert_eq!(path, PathBuf::from("/opt/app/.glucose-dashboard.json"));
    }

    #[test]
    fn exe_dir_used_when_cwd_unknown() {
        // CWD 取不到時，回退到 exe 同目錄。
        let exe = PathBuf::from("/opt/app");
        let path = resolve_config_path(None, None, Some(exe));
        assert_eq!(path, PathBuf::from("/opt/app/.glucose-dashboard.json"));
    }

    #[test]
    fn last_resort_is_relative_cwd_name() {
        // env、cwd、exe 皆無時，回退到相對路徑（與歷史行為一致）。
        let path = resolve_config_path(None, None, None);
        assert_eq!(path, PathBuf::from(".glucose-dashboard.json"));
    }
}
