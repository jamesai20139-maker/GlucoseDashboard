use crate::errors::AppError;

pub async fn begin_browser_flow() -> Result<(), AppError> {
    Err(AppError::NotConfigured(
        "請先設定 OAuth 用戶端，再重新執行設定。".into(),
    ))
}
