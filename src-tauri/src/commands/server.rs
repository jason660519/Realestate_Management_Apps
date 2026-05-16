use std::time::Duration;

use chrono::Utc;
use tauri::State;

use crate::{
    errors::AppError,
    models::{AppConfig, HealthService, ServerHealth},
    state::AppState,
};

#[tauri::command]
pub async fn check_server_health(state: State<'_, AppState>) -> Result<ServerHealth, AppError> {
    let config = state.config()?;

    check_server_health_for_config(config).await
}

pub async fn check_server_health_for_config(config: AppConfig) -> Result<ServerHealth, AppError> {
    let base_url = config
        .server
        .base_url
        .trim()
        .trim_end_matches('/')
        .to_string();

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

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use super::*;

    #[test]
    fn server_health_reports_degraded_for_non_success_response() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = listener
            .local_addr()
            .expect("test server address should resolve");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test request should connect");
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer);
            stream
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .expect("test response should write");
        });

        let mut config = AppConfig::default();
        config.server.base_url = format!("http://{address}");
        config.server.timeout_sec = 2;

        let health = tauri::async_runtime::block_on(check_server_health_for_config(config))
            .expect("health check should complete");
        server.join().expect("test server should exit");

        assert_eq!(health.overall, "degraded");
        assert_eq!(health.services.len(), 1);
        assert_eq!(health.services[0].name, "reverse-proxy");
        assert_eq!(health.services[0].status, "fail");
        assert!(health
            .error
            .expect("degraded response should include an error")
            .contains("503"));
    }

    #[test]
    fn server_health_reports_not_configured_for_empty_base_url() {
        let mut config = AppConfig::default();
        config.server.base_url = "   ".to_string();

        let health = tauri::async_runtime::block_on(check_server_health_for_config(config))
            .expect("health check should complete");

        assert_eq!(health.overall, "not_configured");
        assert_eq!(health.base_url, "");
        assert!(health.services.is_empty());
    }
}
