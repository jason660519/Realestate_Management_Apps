use tauri::State;

use crate::{errors::AppError, models::PluginStatus, state::AppState};

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginStatus>, AppError> {
    let config = state.config()?;

    Ok(vec![
        PluginStatus {
            id: "saydo".to_string(),
            name: "SayDo".to_string(),
            enabled: config.plugins.saydo_enabled,
            permission_scope: vec!["text handoff draft only".to_string()],
            status: if config.plugins.saydo_enabled {
                "degraded".to_string()
            } else {
                "disabled".to_string()
            },
            last_handshake_at: None,
            last_error: None,
        },
        PluginStatus {
            id: "project_manager".to_string(),
            name: "Project-Manager".to_string(),
            enabled: config.plugins.project_manager_enabled,
            permission_scope: vec!["task export pending sync".to_string()],
            status: if config.plugins.project_manager_enabled {
                "degraded".to_string()
            } else {
                "disabled".to_string()
            },
            last_handshake_at: None,
            last_error: None,
        },
    ])
}
