use std::path::PathBuf;

use super::sheet_parser::parse_csv;
use crate::{
    config::service::normalize_sheet_reference,
    domain::{DataQualityIssue, GlucoseRecord},
    errors::AppError,
};

#[derive(Clone, Default)]
pub struct SyncService {
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub fixture_path: Option<PathBuf>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ConnectionReport {
    pub ok: bool,
    pub sheet_id: Option<String>,
    pub sheet_gid: Option<String>,
    pub sheet_name: Option<String>,
    pub url: Option<String>,
    pub http_status: Option<u16>,
    pub record_count: Option<usize>,
    pub issue_count: Option<usize>,
    pub message: String,
    pub detail: Option<String>,
}

impl SyncService {
    pub async fn load(&self) -> Result<(Vec<GlucoseRecord>, Vec<DataQualityIssue>), AppError> {
        let text = if let Some(path) = self.fixture_path.clone() {
            std::fs::read_to_string(path).map_err(|_| AppError::Sync("無法讀取資料來源。".into()))?
        } else {
            let sheet_id = self
                .sheet_id
                .clone()
                .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
            let sheet_gid = self.sheet_gid.clone();
            let sheet_name = self.sheet_name.clone().unwrap_or_else(|| "Sheet1".into());
            let response = fetch_google_sheet_csv(&sheet_id, sheet_gid.as_deref(), &sheet_name).await?;
            if !response.status.is_success() {
                return Err(AppError::Sync(response.message));
            }
            response.body
        };
        Ok(parse_csv(&text))
    }

    pub async fn test_google_sheet_connection(&self) -> Result<ConnectionReport, AppError> {
        let raw_sheet_id = self
            .sheet_id
            .clone()
            .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
        let (sheet_id, parsed_gid) = normalize_sheet_reference(&raw_sheet_id)
            .ok_or_else(|| AppError::NotConfigured("尚未設定 Google Sheet。".into()))?;
        let sheet_gid = self.sheet_gid.clone().or(parsed_gid);
        let sheet_name = self.sheet_name.clone().unwrap_or_else(|| "Sheet1".into());
        let response = fetch_google_sheet_csv(&sheet_id, sheet_gid.as_deref(), &sheet_name).await?;
        let (records, issues) = parse_csv(&response.body);
        let parse_ok = response.status.is_success() && !issues.iter().any(|issue| issue.code == crate::domain::IssueCode::HeaderMismatch);
        Ok(ConnectionReport {
            ok: parse_ok,
            sheet_id: Some(sheet_id),
            sheet_gid,
            sheet_name: Some(sheet_name),
            url: Some(response.url),
            http_status: Some(response.status.as_u16()),
            record_count: Some(records.len()),
            issue_count: Some(issues.len()),
            message: if parse_ok {
                "Google Sheet 可連線".into()
            } else if !response.status.is_success() {
                response.message.clone()
            } else {
                "已連上 Google，但回應內容不是可解析的 Sheet CSV。".into()
            },
            detail: response.detail.or_else(|| {
                if issues.is_empty() {
                    None
                } else {
                    Some(
                        issues
                            .into_iter()
                            .map(|issue| format!("第 {} 列：{}", issue.source_row_number, issue.message_zh_tw))
                            .collect::<Vec<_>>()
                            .join("；"),
                    )
                }
            }),
        })
    }
}

struct GoogleSheetFetch {
    body: String,
    status: reqwest::StatusCode,
    url: String,
    message: String,
    detail: Option<String>,
}

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
    let response = client.get(&url).send().await.map_err(|error| {
        AppError::Sync(format!("無法連線到 Google Sheet：{error}"))
    })?;
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
