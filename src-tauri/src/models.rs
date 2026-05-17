use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub server: ServerConfig,
    pub plugins: PluginConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                base_url: "http://192.168.1.6:8080".to_string(),
                health_check_interval_sec: 30,
                timeout_sec: 10,
            },
            plugins: PluginConfig {
                saydo_enabled: false,
                project_manager_enabled: false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub base_url: String,
    pub health_check_interval_sec: u64,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfig {
    pub saydo_enabled: bool,
    pub project_manager_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigPatch {
    pub server_base_url: Option<String>,
    pub saydo_enabled: Option<bool>,
    pub project_manager_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDiagnostics {
    pub app_data_dir: String,
    pub config_path: String,
    pub config_exists: bool,
    pub config_readable: bool,
    pub config_file_bytes: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHealth {
    pub overall: String,
    pub checked_at: DateTime<Utc>,
    pub base_url: String,
    pub services: Vec<HealthService>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthService {
    pub name: String,
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub permission_scope: Vec<String>,
    pub status: String,
    pub last_handshake_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}
