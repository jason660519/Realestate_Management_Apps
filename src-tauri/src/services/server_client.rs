use std::time::Duration;

use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{
    errors::AppError,
    models::{AppConfig, HealthService, ServerHealth},
};

/// Single HTTP egress for the desktop app.
///
/// All outbound requests to the internal server (PostgREST today, axum services later)
/// must go through `ServerClient`. WebView code never calls `fetch` directly; Rust
/// command handlers obtain a client per invocation from `AppConfig`.
pub struct ServerClient {
    http: reqwest::Client,
    base_url: String,
}

impl ServerClient {
    /// Build a client from the live `AppConfig`. Returns `None` when the configured
    /// base URL is blank — callers should surface a `not_configured` state instead of
    /// pretending a request was attempted.
    pub fn from_config(config: &AppConfig) -> Result<Option<Self>, AppError> {
        let base_url = config
            .server
            .base_url
            .trim()
            .trim_end_matches('/')
            .to_string();

        if base_url.is_empty() {
            return Ok(None);
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.server.timeout_sec))
            .build()
            .map_err(|error| AppError::ServerUnreachable {
                message: error.to_string(),
            })?;

        Ok(Some(Self { http, base_url }))
    }

    /// GET `<base>/<path>` and deserialize the response body as JSON.
    ///
    /// `path` should already include any leading `/` and query string (e.g.
    /// `"/api/properties?select=id,display_name&limit=100"`). Non-2xx responses
    /// become `AppError::ServerUnreachable` carrying the status code; transport
    /// errors carry the underlying reqwest message.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let url = format!("{}{}", self.base_url, path);

        let response =
            self.http
                .get(&url)
                .send()
                .await
                .map_err(|error| AppError::ServerUnreachable {
                    message: error.to_string(),
                })?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::ServerUnreachable {
                message: format!("HTTP {status} from {url}"),
            });
        }

        response
            .json::<T>()
            .await
            .map_err(|error| AppError::ServerUnreachable {
                message: format!("failed to parse JSON from {url}: {error}"),
            })
    }

    /// POST `<base>/<path>` with a JSON body and deserialize the response.
    ///
    /// Used for write-path RPCs (`/api/rpc/save_property`, …) that route to the
    /// Rust axum service per ADR-010. The body serializes to JSON; non-2xx
    /// responses carry the status code and any plain-text body the server
    /// returned so the UI can surface a precise error reason.
    pub async fn post_json<TIn, TOut>(&self, path: &str, body: &TIn) -> Result<TOut, AppError>
    where
        TIn: Serialize + ?Sized,
        TOut: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|error| AppError::ServerUnreachable {
                message: error.to_string(),
            })?;

        let status = response.status();
        if !status.is_success() {
            let body_snippet = response.text().await.unwrap_or_default();
            let trimmed = body_snippet.trim();
            let suffix = if trimmed.is_empty() {
                String::new()
            } else {
                format!(": {trimmed}")
            };
            return Err(AppError::ServerUnreachable {
                message: format!("HTTP {status} from {url}{suffix}"),
            });
        }

        response
            .json::<TOut>()
            .await
            .map_err(|error| AppError::ServerUnreachable {
                message: format!("failed to parse JSON from {url}: {error}"),
            })
    }

    /// Probe `/health` and translate the result into a `ServerHealth` domain value.
    ///
    /// Returns one of `ok` / `degraded` / `offline`. The empty-base-url case lives in
    /// `check_health_for_config` so callers don't need to construct a client just to
    /// report `not_configured`.
    pub async fn check_health(&self) -> ServerHealth {
        let started = std::time::Instant::now();
        let response = self
            .http
            .get(format!("{}/health", self.base_url))
            .send()
            .await;

        match response {
            Ok(response) if response.status().is_success() => ServerHealth {
                overall: "ok".to_string(),
                checked_at: Utc::now(),
                base_url: self.base_url.clone(),
                services: vec![HealthService {
                    name: "reverse-proxy".to_string(),
                    status: "ok".to_string(),
                    latency_ms: Some(started.elapsed().as_millis() as u64),
                    error: None,
                }],
                error: None,
            },
            Ok(response) => ServerHealth {
                overall: "degraded".to_string(),
                checked_at: Utc::now(),
                base_url: self.base_url.clone(),
                services: vec![HealthService {
                    name: "reverse-proxy".to_string(),
                    status: "fail".to_string(),
                    latency_ms: Some(started.elapsed().as_millis() as u64),
                    error: Some(format!("HTTP {}", response.status())),
                }],
                error: Some(format!("Health endpoint returned {}", response.status())),
            },
            Err(error) => ServerHealth {
                overall: "offline".to_string(),
                checked_at: Utc::now(),
                base_url: self.base_url.clone(),
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
            },
        }
    }
}

/// Convenience entrypoint used by the server command. Handles the `not_configured`
/// case before even constructing a client, mirroring the pre-refactor behaviour.
pub async fn check_health_for_config(config: AppConfig) -> Result<ServerHealth, AppError> {
    match ServerClient::from_config(&config)? {
        None => Ok(ServerHealth {
            overall: "not_configured".to_string(),
            checked_at: Utc::now(),
            base_url: String::new(),
            services: Vec::new(),
            error: Some("Server base URL is not configured".to_string()),
        }),
        Some(client) => Ok(client.check_health().await),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Probe {
        ok: bool,
        message: String,
    }

    fn spawn_test_server<F>(handler: F) -> std::net::SocketAddr
    where
        F: FnOnce(&[u8]) -> Vec<u8> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = listener.local_addr().expect("test address should resolve");

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test request should connect");
            let mut buffer = [0u8; 4096];
            let read = stream.read(&mut buffer).unwrap_or(0);
            let response = handler(&buffer[..read]);
            stream
                .write_all(&response)
                .expect("test response should write");
        });

        address
    }

    fn config_for(address: std::net::SocketAddr) -> AppConfig {
        let mut config = AppConfig::default();
        config.server.base_url = format!("http://{address}");
        config.server.timeout_sec = 2;
        config
    }

    #[test]
    fn get_json_deserializes_success_response() {
        let address = spawn_test_server(|_| {
            let body = br#"{"ok":true,"message":"hi"}"#;
            let mut out = Vec::new();
            out.extend_from_slice(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ",
            );
            out.extend_from_slice(body.len().to_string().as_bytes());
            out.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
            out.extend_from_slice(body);
            out
        });

        let client = ServerClient::from_config(&config_for(address))
            .expect("client should build")
            .expect("client should be configured");

        let probe = tauri::async_runtime::block_on(client.get_json::<Probe>("/probe"))
            .expect("probe should succeed");
        assert_eq!(
            probe,
            Probe {
                ok: true,
                message: "hi".to_string()
            }
        );
    }

    #[test]
    fn get_json_surfaces_non_success_status() {
        let address = spawn_test_server(|_| {
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec()
        });

        let client = ServerClient::from_config(&config_for(address))
            .expect("client should build")
            .expect("client should be configured");

        let result = tauri::async_runtime::block_on(client.get_json::<Probe>("/probe"));
        let error = result.expect_err("503 should be an error");
        let AppError::ServerUnreachable { message } = error else {
            panic!("expected ServerUnreachable, got {error:?}");
        };
        assert!(message.contains("503"));
    }

    #[test]
    fn check_health_reports_degraded_for_non_success_response() {
        let address = spawn_test_server(|_| {
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec()
        });

        let client = ServerClient::from_config(&config_for(address))
            .expect("client should build")
            .expect("client should be configured");

        let health = tauri::async_runtime::block_on(client.check_health());
        assert_eq!(health.overall, "degraded");
        assert_eq!(health.services.len(), 1);
        assert_eq!(health.services[0].status, "fail");
        assert!(health
            .error
            .expect("degraded response should include an error")
            .contains("503"));
    }

    #[test]
    fn check_health_for_config_reports_not_configured_for_empty_base_url() {
        let mut config = AppConfig::default();
        config.server.base_url = "   ".to_string();

        let health = tauri::async_runtime::block_on(check_health_for_config(config))
            .expect("health check should complete");

        assert_eq!(health.overall, "not_configured");
        assert_eq!(health.base_url, "");
        assert!(health.services.is_empty());
    }

    #[derive(Debug, serde::Serialize)]
    struct EchoRequest {
        message: String,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct EchoResponse {
        echoed: String,
    }

    #[test]
    fn post_json_sends_body_and_parses_response() {
        let address = spawn_test_server(|raw_request| {
            let request = std::str::from_utf8(raw_request).unwrap_or_default();
            // Sanity: body should be the JSON we sent.
            assert!(
                request.contains("\"message\":\"hello\""),
                "expected POST body to contain message:hello, got {request:?}"
            );
            assert!(
                request.starts_with("POST /api/rpc/echo"),
                "expected POST /api/rpc/echo, got {request:?}"
            );

            let body = br#"{"echoed":"hello"}"#;
            let mut out = Vec::new();
            out.extend_from_slice(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ",
            );
            out.extend_from_slice(body.len().to_string().as_bytes());
            out.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
            out.extend_from_slice(body);
            out
        });

        let client = ServerClient::from_config(&config_for(address))
            .expect("client should build")
            .expect("client should be configured");

        let response = tauri::async_runtime::block_on(client.post_json::<_, EchoResponse>(
            "/api/rpc/echo",
            &EchoRequest {
                message: "hello".to_string(),
            },
        ))
        .expect("post should succeed");

        assert_eq!(
            response,
            EchoResponse {
                echoed: "hello".to_string()
            }
        );
    }

    #[test]
    fn post_json_surfaces_4xx_status_with_body_snippet() {
        let address = spawn_test_server(|_| {
            let body = br#"{"kind":"validation","message":"display_name is required"}"#;
            let mut out = Vec::new();
            out.extend_from_slice(
                b"HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: ",
            );
            out.extend_from_slice(body.len().to_string().as_bytes());
            out.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
            out.extend_from_slice(body);
            out
        });

        let client = ServerClient::from_config(&config_for(address))
            .expect("client should build")
            .expect("client should be configured");

        let result = tauri::async_runtime::block_on(client.post_json::<_, EchoResponse>(
            "/api/rpc/echo",
            &EchoRequest {
                message: "hello".to_string(),
            },
        ));
        let error = result.expect_err("4xx should error");
        let AppError::ServerUnreachable { message } = error else {
            panic!("expected ServerUnreachable, got {error:?}");
        };
        assert!(message.contains("400"));
        assert!(
            message.contains("display_name is required"),
            "expected body snippet in error, got {message:?}"
        );
    }
}
