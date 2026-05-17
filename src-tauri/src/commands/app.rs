use tauri::State;

use crate::{
    errors::AppError,
    models::{AppConfig, AppConfigPatch, StorageDiagnostics},
    state::AppState,
};

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, AppError> {
    state.config()
}

#[tauri::command]
pub fn update_app_config(
    state: State<'_, AppState>,
    patch: AppConfigPatch,
) -> Result<AppConfig, AppError> {
    state.update_config(patch)
}

#[tauri::command]
pub fn get_storage_diagnostics(state: State<'_, AppState>) -> Result<StorageDiagnostics, AppError> {
    Ok(state.storage_diagnostics())
}
