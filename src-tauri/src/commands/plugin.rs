use tauri::State;

use crate::{
    errors::AppError,
    models::{AppConfig, PluginStatus},
    state::AppState,
};

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginStatus>, AppError> {
    let config = state.config()?;

    Ok(plugin_statuses(&config))
}

pub fn plugin_statuses(config: &AppConfig) -> Vec<PluginStatus> {
    vec![
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_statuses_reflect_configured_plugin_flags() {
        let mut config = AppConfig::default();
        config.plugins.saydo_enabled = true;

        let plugins = plugin_statuses(&config);
        let saydo = plugins
            .iter()
            .find(|plugin| plugin.id == "saydo")
            .expect("SayDo plugin should exist");
        let project_manager = plugins
            .iter()
            .find(|plugin| plugin.id == "project_manager")
            .expect("Project-Manager plugin should exist");

        assert!(saydo.enabled);
        assert_eq!(saydo.status, "degraded");
        assert!(!project_manager.enabled);
        assert_eq!(project_manager.status, "disabled");
    }
}
