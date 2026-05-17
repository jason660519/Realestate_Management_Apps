use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    errors::AppError,
    models::{AppConfig, PluginConfig, ServerConfig, StorageDiagnostics},
};

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn from_app_data_dir(app_data_dir: PathBuf) -> Self {
        Self {
            config_path: app_data_dir.join(CONFIG_FILE_NAME),
        }
    }

    #[cfg(test)]
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn load_or_create(&self) -> Result<AppConfig, AppError> {
        if !self.config_path.exists() {
            let config = AppConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        let content =
            fs::read_to_string(&self.config_path).map_err(|error| AppError::ConfigStorage {
                message: format!(
                    "Failed to read config file {}: {error}",
                    self.config_path.display()
                ),
            })?;

        let stored: StoredAppConfig =
            toml::from_str(&content).map_err(|error| AppError::ConfigStorage {
                message: format!(
                    "Failed to parse config file {}: {error}",
                    self.config_path.display()
                ),
            })?;

        Ok(stored.into())
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), AppError> {
        ensure_parent_dir(&self.config_path)?;

        let stored = StoredAppConfig::from(config.clone());
        let content = toml::to_string_pretty(&stored).map_err(|error| AppError::ConfigStorage {
            message: format!("Failed to serialize config: {error}"),
        })?;

        let tmp_path = self.config_path.with_extension("toml.tmp");
        fs::write(&tmp_path, content).map_err(|error| AppError::ConfigStorage {
            message: format!(
                "Failed to write temporary config file {}: {error}",
                tmp_path.display()
            ),
        })?;

        fs::rename(&tmp_path, &self.config_path).map_err(|error| AppError::ConfigStorage {
            message: format!(
                "Failed to replace config file {}: {error}",
                self.config_path.display()
            ),
        })
    }

    pub fn diagnostics(&self) -> StorageDiagnostics {
        let app_data_dir = self
            .config_path
            .parent()
            .map_or_else(String::new, |path| path.display().to_string());

        match fs::metadata(&self.config_path) {
            Ok(metadata) => StorageDiagnostics {
                app_data_dir,
                config_path: self.config_path.display().to_string(),
                config_exists: true,
                config_readable: fs::read_to_string(&self.config_path).is_ok(),
                config_file_bytes: Some(metadata.len()),
                error: None,
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => StorageDiagnostics {
                app_data_dir,
                config_path: self.config_path.display().to_string(),
                config_exists: false,
                config_readable: false,
                config_file_bytes: None,
                error: None,
            },
            Err(error) => StorageDiagnostics {
                app_data_dir,
                config_path: self.config_path.display().to_string(),
                config_exists: false,
                config_readable: false,
                config_file_bytes: None,
                error: Some(error.to_string()),
            },
        }
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| AppError::ConfigStorage {
            message: format!(
                "Failed to create config directory {}: {error}",
                parent.display()
            ),
        })?;
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredAppConfig {
    #[serde(default)]
    server: StoredServerConfig,
    #[serde(default)]
    plugins: StoredPluginConfig,
}

impl From<AppConfig> for StoredAppConfig {
    fn from(config: AppConfig) -> Self {
        Self {
            server: StoredServerConfig {
                base_url: config.server.base_url,
                health_check_interval_sec: config.server.health_check_interval_sec,
                timeout_sec: config.server.timeout_sec,
            },
            plugins: StoredPluginConfig {
                saydo_enabled: config.plugins.saydo_enabled,
                project_manager_enabled: config.plugins.project_manager_enabled,
            },
        }
    }
}

impl From<StoredAppConfig> for AppConfig {
    fn from(config: StoredAppConfig) -> Self {
        Self {
            server: ServerConfig {
                base_url: config.server.base_url,
                health_check_interval_sec: config.server.health_check_interval_sec,
                timeout_sec: config.server.timeout_sec,
            },
            plugins: PluginConfig {
                saydo_enabled: config.plugins.saydo_enabled,
                project_manager_enabled: config.plugins.project_manager_enabled,
            },
        }
    }
}

impl Default for StoredAppConfig {
    fn default() -> Self {
        AppConfig::default().into()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredServerConfig {
    base_url: String,
    health_check_interval_sec: u64,
    timeout_sec: u64,
}

impl Default for StoredServerConfig {
    fn default() -> Self {
        StoredAppConfig::default().server
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredPluginConfig {
    saydo_enabled: bool,
    project_manager_enabled: bool,
}

impl Default for StoredPluginConfig {
    fn default() -> Self {
        StoredAppConfig::default().plugins
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn load_or_create_writes_default_snake_case_toml() {
        let store = ConfigStore::new(unique_config_path("default"));

        let config = store.load_or_create().expect("default config should load");
        let content =
            fs::read_to_string(&store.config_path).expect("config file should be created");

        assert_eq!(config.server.base_url, AppConfig::default().server.base_url);
        assert!(content.contains("[server]"));
        assert!(content.contains("base_url"));
        assert!(content.contains("project_manager_enabled"));
    }

    #[test]
    fn save_and_reload_preserves_config_values() {
        let store = ConfigStore::new(unique_config_path("roundtrip"));
        let mut config = AppConfig::default();
        config.server.base_url = "http://127.0.0.1:8080".to_string();
        config.plugins.saydo_enabled = true;
        config.plugins.project_manager_enabled = true;

        store.save(&config).expect("config should save");
        let reloaded = store.load_or_create().expect("config should reload");

        assert_eq!(reloaded.server.base_url, "http://127.0.0.1:8080");
        assert!(reloaded.plugins.saydo_enabled);
        assert!(reloaded.plugins.project_manager_enabled);
    }

    #[test]
    fn diagnostics_reports_created_config_file() {
        let store = ConfigStore::new(unique_config_path("diagnostics"));

        store
            .load_or_create()
            .expect("default config should be created");
        let diagnostics = store.diagnostics();

        assert!(diagnostics.config_exists);
        assert!(diagnostics.config_readable);
        assert!(diagnostics.config_file_bytes.unwrap_or_default() > 0);
        assert!(diagnostics.error.is_none());
        assert!(diagnostics.config_path.ends_with(CONFIG_FILE_NAME));
    }

    fn unique_config_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();

        std::env::temp_dir()
            .join("realestate-management-apps-tests")
            .join(format!("{label}-{nanos}"))
            .join(CONFIG_FILE_NAME)
    }
}
