use std::path::PathBuf;

use super::sheet_parser::parse_csv;
use crate::{
    domain::{CustomEvent, DashboardTableRow, DataQualityIssue, GlucoseRecord},
    errors::AppError,
};

/// Google Sheet 單次抓取結果。供設定載入與資料載入共用。
#[derive(Clone, Debug)]
pub struct GoogleSheetFetch {
    pub body: String,
    pub status: reqwest::StatusCode,
    pub url: String,
    pub message: String,
    pub detail: Option<String>,
}

/// 可注入的 Google Sheet 抓取器。生產環境用 `ReqwestGoogleSheetFetcher`，
/// 測試用 fake 實作（對應工作表名/gid 回傳預先準備的 CSV 字串），讓 handler
/// 與 settings_loader 測試不需連線 Google。
///
/// 使用 boxed future（非 async-trait）以避免新增依賴；回傳的 Future 借用
/// `&self` 與傳入字串（生命週期 `'a`）。
pub trait GoogleSheetFetcher: Send + Sync {
    /// 依 sheet_id + (gid 或 sheet_name) 抓取一個工作表的 CSV 文字。
    /// `sheet_gid` 優先（走 `/export?format=csv&gid=...`）；否則以 `sheet_name`
    /// 走 gviz（`/gviz/tq?tqx=out:csv&sheet=...`）。
    fn fetch_csv<'a>(
        &'a self,
        sheet_id: &'a str,
        sheet_gid: Option<&'a str>,
        sheet_name: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GoogleSheetFetch, AppError>> + Send + 'a>,
    >;
}

/// 生產環境的 reqwest 抓取器，沿用原 `fetch_google_sheet_csv` 的 URL 與行為。
#[derive(Clone, Default)]
pub struct ReqwestGoogleSheetFetcher;

impl GoogleSheetFetcher for ReqwestGoogleSheetFetcher {
    fn fetch_csv<'a>(
        &'a self,
        sheet_id: &'a str,
        sheet_gid: Option<&'a str>,
        sheet_name: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GoogleSheetFetch, AppError>> + Send + 'a>,
    > {
        Box::pin(async move { fetch_google_sheet_csv(sheet_id, sheet_gid, sheet_name).await })
    }
}

#[derive(Clone, Default)]
pub struct SyncService {
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<PathBuf>,
    pub custom_events: Vec<CustomEvent>,
}

impl SyncService {
    /// 載入資料工作表並解析。`fetcher` 注入以利測試；生產呼叫端用
    /// `ReqwestGoogleSheetFetcher`（可用 `load()` 包裝）。fixture 路徑不走網路。
    pub async fn load_with_fetcher(
        &self,
        fetcher: &dyn GoogleSheetFetcher,
    ) -> Result<
        (
            Vec<GlucoseRecord>,
            Vec<DataQualityIssue>,
            Vec<DashboardTableRow>,
        ),
        AppError,
    > {
        // Google Sheet 為唯一正式資料來源：一旦設定 sheet_id 就一律直讀 Sheet，
        // fixture_path 僅在尚未設定 Sheet 時作為開發/離線回退使用。
        let text = if self
            .sheet_id
            .as_ref()
            .map(|id| !id.trim().is_empty())
            .unwrap_or(false)
        {
            let sheet_id = self.sheet_id.clone().unwrap();
            let sheet_gid = self.sheet_gid.clone();
            let sheet_name = self.sheet_name.clone().unwrap_or_else(|| "Sheet1".into());
            let response = fetcher
                .fetch_csv(&sheet_id, sheet_gid.as_deref(), &sheet_name)
                .await?;
            if !response.status.is_success() {
                return Err(AppError::Sync(response.message));
            }
            response.body
        } else if let Some(path) = self.fixture_path.clone() {
            std::fs::read_to_string(path)
                .map_err(|_| AppError::Sync("無法讀取資料來源。".into()))?
        } else {
            return Err(AppError::NotConfigured("尚未設定 Google Sheet。".into()));
        };
        Ok(parse_csv(&text, &self.custom_events))
    }

    /// 生產環境便捷包裝：用 `ReqwestGoogleSheetFetcher` 載入資料表。
    /// 保留為公共 API；路由層已改用 `load_with_fetcher` 以注入測試 fetcher。
    #[allow(dead_code)]
    pub async fn load(
        &self,
    ) -> Result<
        (
            Vec<GlucoseRecord>,
            Vec<DataQualityIssue>,
            Vec<DashboardTableRow>,
        ),
        AppError,
    > {
        self.load_with_fetcher(&ReqwestGoogleSheetFetcher).await
    }
}

/// 抓取單一 Google 工作表 CSV。`sheet_gid` 優先走 export URL；否則以 `sheet_name`
/// 走 gviz URL。沿用既有 URL 規則。
async fn fetch_google_sheet_csv(
    sheet_id: &str,
    sheet_gid: Option<&str>,
    sheet_name: &str,
) -> Result<GoogleSheetFetch, AppError> {
    let url = if let Some(gid) = sheet_gid {
        format!("https://docs.google.com/spreadsheets/d/{sheet_id}/export?format=csv&gid={gid}")
    } else {
        let encoded_sheet_name = urlencoding::encode(sheet_name);
        format!(
            "https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet={encoded_sheet_name}"
        )
    };
    let client = reqwest::Client::builder()
        .user_agent("GlucoseDashboard/0.1")
        .build()
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| AppError::Sync(format!("無法連線到 Google Sheet：{error}")))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| AppError::Sync(format!("無法讀取 Google Sheet 內容：{error}")))?;
    let message = if status.is_success() {
        "Google Sheet 可連線".into()
    } else {
        format!("Google Sheet 讀取失敗，HTTP 狀態碼 {}。", status)
    };
    let detail_text: String = body.chars().take(320).collect();
    let detail = if detail_text.is_empty() {
        None
    } else {
        Some(detail_text.replace('\n', " "))
    };
    Ok(GoogleSheetFetch {
        body,
        status,
        url,
        message,
        detail,
    })
}
