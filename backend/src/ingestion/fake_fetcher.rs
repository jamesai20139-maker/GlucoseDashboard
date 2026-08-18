//! 測試專用的假 Google Sheet 抓取器。依工作表名/gid 回傳預先準備的 CSV 字串，
//! 並記錄呼叫計數，讓 handler 測試驗證「每次載入都抓設定」等行為，無需連線 Google。
#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::sync_service::{GoogleSheetFetch, GoogleSheetFetcher};
use crate::errors::AppError;

/// 假抓取器：以 `(sheet_id, gid_or_name)` 為 key 對應 CSV 文字。
/// `call_count` 記錄總抓取次數（含資料表與設定表）；`name_calls` 記錄 by-name 抓取。
#[derive(Clone, Default)]
pub struct FakeFetcher {
    /// key = gid（優先）或 sheet_name，value = CSV body。
    pub responses: Arc<Mutex<HashMap<String, String>>>,
    /// 累計 fetch 呼叫次數。
    pub call_count: Arc<Mutex<usize>>,
    /// by-name 抓取的呼叫次數（設定工作表用）。
    pub name_calls: Arc<Mutex<Vec<String>>>,
}

impl FakeFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// 設定某工作表的回應（以名稱為 key；設定表皆 by-name）。
    pub fn with_name(self, name: &str, body: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(name.to_string(), body.to_string());
        self
    }

    /// 設定某 gid 的回應（資料表用）。
    pub fn with_gid(self, gid: &str, body: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(gid.to_string(), body.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn total_calls(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn name_call_count(&self) -> usize {
        self.name_calls.lock().unwrap().len()
    }

    #[allow(dead_code)]
    pub fn name_calls(&self) -> Vec<String> {
        self.name_calls.lock().unwrap().clone()
    }
}

impl GoogleSheetFetcher for FakeFetcher {
    fn fetch_csv<'a>(
        &'a self,
        _sheet_id: &'a str,
        sheet_gid: Option<&'a str>,
        sheet_name: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GoogleSheetFetch, AppError>> + Send + 'a>,
    > {
        let responses = self.responses.clone();
        let call_count = self.call_count.clone();
        let name_calls = self.name_calls.clone();
        Box::pin(async move {
            *call_count.lock().unwrap() += 1;
            name_calls.lock().unwrap().push(sheet_name.to_string());
            let key = sheet_gid.unwrap_or(sheet_name);
            let body = responses
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .unwrap_or_default();
            Ok(GoogleSheetFetch {
                body,
                status: reqwest::StatusCode::OK,
                url: format!("fake://{}", key),
                message: "fake".into(),
                detail: None,
            })
        })
    }
}
