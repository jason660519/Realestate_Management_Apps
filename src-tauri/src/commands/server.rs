use tauri::State;

use crate::{errors::AppError, models::ServerHealth, services::server_client, state::AppState};

#[tauri::command]
pub async fn check_server_health(state: State<'_, AppState>) -> Result<ServerHealth, AppError> {
    let config = state.config()?;
    server_client::check_health_for_config(config).await
}
