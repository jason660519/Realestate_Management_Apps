use std::sync::Mutex;

use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{AppConfig, AppConfigPatch, StorageDiagnostics},
    services::config::ConfigStore,
};

pub struct AppState {
    config: Mutex<AppConfig>,
    store: ConfigStore,
    local_db: SqlitePool,
}

impl AppState {
    pub fn load(store: ConfigStore, local_db: SqlitePool) -> Result<Self, AppError> {
        let config = store.load_or_create()?;

        Ok(Self {
            config: Mutex::new(config),
            store,
            local_db,
        })
    }

    pub fn config(&self) -> Result<AppConfig, AppError> {
        let config = self.config.lock().map_err(|error| AppError::State {
            message: error.to_string(),
        })?;

        Ok(config.clone())
    }

    pub fn local_db(&self) -> &SqlitePool {
        &self.local_db
    }

    pub fn update_config(&self, patch: AppConfigPatch) -> Result<AppConfig, AppError> {
        let mut config = self.config.lock().map_err(|error| AppError::State {
            message: error.to_string(),
        })?;

        let mut next_config = config.clone();

        if let Some(server_base_url) = patch.server_base_url {
            let trimmed = server_base_url.trim();
            if trimmed.is_empty() {
                return Err(AppError::InvalidInput {
                    message: "Server base URL cannot be empty".to_string(),
                });
            }

            next_config.server.base_url = trimmed.to_string();
        }

        if let Some(saydo_enabled) = patch.saydo_enabled {
            next_config.plugins.saydo_enabled = saydo_enabled;
        }

        if let Some(project_manager_enabled) = patch.project_manager_enabled {
            next_config.plugins.project_manager_enabled = project_manager_enabled;
        }

        self.store.save(&next_config)?;
        *config = next_config.clone();

        Ok(next_config)
    }

    pub fn storage_diagnostics(&self) -> StorageDiagnostics {
        self.store.diagnostics()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::services::local_db;

    #[tokio::test]
    async fn update_config_persists_patch_to_config_file() {
        let store = ConfigStore::new(unique_config_path("patch"));
        let pool = local_db::open_in_memory().await;
        let state = AppState::load(store.clone(), pool).expect("state should load");

        let updated = state
            .update_config(AppConfigPatch {
                server_base_url: Some("  http://127.0.0.1:9090/  ".to_string()),
                saydo_enabled: Some(true),
                project_manager_enabled: Some(true),
            })
            .expect("config patch should persist");

        assert_eq!(updated.server.base_url, "http://127.0.0.1:9090/");
        assert!(updated.plugins.saydo_enabled);
        assert!(updated.plugins.project_manager_enabled);

        let reloaded = store
            .load_or_create()
            .expect("persisted config should reload");
        assert_eq!(reloaded, updated);
    }

    #[tokio::test]
    async fn update_config_rejects_empty_server_base_url() {
        let store = ConfigStore::new(unique_config_path("invalid"));
        let pool = local_db::open_in_memory().await;
        let state = AppState::load(store, pool).expect("state should load");

        let result = state.update_config(AppConfigPatch {
            server_base_url: Some("   ".to_string()),
            saydo_enabled: None,
            project_manager_enabled: None,
        });

        assert!(matches!(result, Err(AppError::InvalidInput { .. })));
    }

    fn unique_config_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();

        std::env::temp_dir()
            .join("realestate-management-apps-state-tests")
            .join(format!("{label}-{nanos}"))
            .join("config.toml")
    }
}
