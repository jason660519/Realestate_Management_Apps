use tauri::State;

use crate::{
    errors::AppError,
    models::{AppConfig, AppConfigPatch},
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
