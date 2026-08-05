use crate::{
    config::{model::LocalConfiguration, store::ConfigStore},
    errors::AppError,
};

pub fn configure(
    store: &ConfigStore,
    sheet_id: String,
    sheet_name: String,
    fixture_path: Option<String>,
) -> Result<LocalConfiguration, AppError> {
    if sheet_id.trim().is_empty() {
        return Err(AppError::Invalid("Google Sheet ID 不可為空白。".into()));
    }
    let config = LocalConfiguration {
        schema_version: 1,
        sheet_id: Some(sheet_id),
        sheet_name: Some(sheet_name),
        fixture_path,
        credential_reference: None,
        last_successful_sync_at: None,
    };
    store.save(&config).map_err(AppError::Internal)?;
    Ok(config)
}
