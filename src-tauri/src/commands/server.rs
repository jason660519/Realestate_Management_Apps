use std::time::Duration;

use chrono::Utc;
use tauri::State;

use crate::{
    errors::AppError,
    models::{HealthService, ServerHealth},
    state::AppState,
};

#[tauri::command]
pub async fn check_server_health(state: State<'_, AppState>) -> Result<ServerHealth, AppError> {
    let config = state.config()?;
    let base_url = config.server.base_url.trim_end_matches('/').to_string();

    if base_url.is_empty() {
        return Ok(ServerHealth {
            overall: "not_configured".to_string(),
            checked_at: Utc::now(),
            base_url,
            services: Vec::new(),
            error: Some("Server base URL is not configured".to_string()),
        });
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.server.timeout_sec))
        .build()
        .map_err(|error| AppError::ServerUnreachable {
            message: error.to_string(),
        })?;

    let started = std::time::Instant::now();
    let response = client.get(format!("{base_url}/health")).send().await;

    match response {
        Ok(response) if response.status().is_success() => Ok(ServerHealth {
            overall: "ok".to_string(),
            checked_at: Utc::now(),
            base_url,
            services: vec![HealthService {
                name: "reverse-proxy".to_string(),
                status: "ok".to_string(),
                latency_ms: Some(started.elapsed().as_millis() as u64),
                error: None,
            }],
            error: None,
        }),
        Ok(response) => Ok(ServerHealth {
            overall: "degraded".to_string(),
            checked_at: Utc::now(),
            base_url,
            services: vec![HealthService {
                name: "reverse-proxy".to_string(),
                status: "fail".to_string(),
                latency_ms: Some(started.elapsed().as_millis() as u64),
                error: Some(format!("HTTP {}", response.status())),
            }],
            error: Some(format!("Health endpoint returned {}", response.status())),
        }),
        Err(error) => Ok(ServerHealth {
            overall: "offline".to_string(),
            checked_at: Utc::now(),
            base_url,
            services: vec![
                HealthService {
                    name: "postgres".to_string(),
                    status: "not_configured".to_string(),
                    latency_ms: None,
                    error: None,
                },
                HealthService {
                    name: "postgrest".to_string(),
                    status: "not_configured".to_string(),
                    latency_ms: None,
                    error: None,
                },
            ],
            error: Some(error.to_string()),
        }),
    }
}
