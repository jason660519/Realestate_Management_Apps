use std::sync::Mutex;

use crate::{
    errors::AppError,
    models::{AppConfig, AppConfigPatch},
};

pub struct AppState {
    config: Mutex<AppConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
        }
    }
}

impl AppState {
    pub fn config(&self) -> Result<AppConfig, AppError> {
        let config = self.config.lock().map_err(|error| AppError::State {
            message: error.to_string(),
        })?;

        Ok(config.clone())
    }

    pub fn update_config(&self, patch: AppConfigPatch) -> Result<AppConfig, AppError> {
        let mut config = self.config.lock().map_err(|error| AppError::State {
            message: error.to_string(),
        })?;

        if let Some(server_base_url) = patch.server_base_url {
            let trimmed = server_base_url.trim();
            if trimmed.is_empty() {
                return Err(AppError::InvalidInput {
                    message: "Server base URL cannot be empty".to_string(),
                });
            }

            config.server.base_url = trimmed.to_string();
        }

        if let Some(saydo_enabled) = patch.saydo_enabled {
            config.plugins.saydo_enabled = saydo_enabled;
        }

        if let Some(project_manager_enabled) = patch.project_manager_enabled {
            config.plugins.project_manager_enabled = project_manager_enabled;
        }

        Ok(config.clone())
    }
}
