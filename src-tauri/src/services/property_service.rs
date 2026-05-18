use chrono::Utc;
use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{
        AppConfig, PropertyMutationResponse, PropertySource, PropertySummariesResult,
        PropertySummary, SavePropertyPayload,
    },
    services::{property_cache, server_client::ServerClient},
};

/// PostgREST `select` projection for the property summary surface. Keep the column
/// list in sync with `PropertySummary` — adding or renaming a field requires updating
/// both this constant and the model.
const SUMMARY_SELECT: &str = "id,display_name,kind,status,address_raw,updated_at";
const SUMMARY_DEFAULT_LIMIT: u32 = 100;

/// Fetch a read-only batch of property summaries.
///
/// Write-through cache: on a successful server fetch, replace the local
/// `property_summary_cache` so a later offline run shows the same view marked
/// stale. On failure (server unreachable, parse error, blank base URL), fall
/// back to whatever the cache holds — the UI surfaces `source` + `error` so
/// the operator can tell why they are looking at potentially old data.
pub async fn list_property_summaries(
    config: &AppConfig,
    pool: &SqlitePool,
) -> Result<PropertySummariesResult, AppError> {
    let Some(client) = ServerClient::from_config(config)? else {
        return fallback_to_cache(
            pool,
            "Server URL is not configured; showing cached property list.",
        )
        .await;
    };

    let path = format!(
        "/api/properties?select={SUMMARY_SELECT}&order=updated_at.desc&limit={SUMMARY_DEFAULT_LIMIT}"
    );

    match client.get_json::<Vec<PropertySummary>>(&path).await {
        Ok(rows) => {
            let synced_at = Utc::now();
            // Cache write must not fail the live response — log via Err shape but
            // surface the server data to the UI. A degraded cache is preferable
            // to losing a successful fetch.
            if let Err(error) = property_cache::replace_summaries(pool, &rows, synced_at).await {
                eprintln!("Property cache write failed (continuing with live data): {error}");
            }

            Ok(PropertySummariesResult {
                rows,
                source: PropertySource::Live,
                last_synced_at: Some(synced_at),
                error: None,
            })
        }
        Err(error) => fallback_to_cache(pool, &error.to_string()).await,
    }
}

/// Skeleton save path (ADR-010). Posts to `/api/rpc/save_property` on the axum
/// service via the shared `ServerClient`. Returns `InvalidInput` rather than
/// pretending success when the server URL is blank — saves must never fall
/// back to a cache-only path because the server is canonical for confirmed
/// data.
pub async fn save_property(
    config: &AppConfig,
    payload: &SavePropertyPayload,
) -> Result<PropertyMutationResponse, AppError> {
    if payload.display_name.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: "display_name is required".to_string(),
        });
    }

    let Some(client) = ServerClient::from_config(config)? else {
        return Err(AppError::InvalidInput {
            message: "Server URL must be configured before saving a property".to_string(),
        });
    };

    client
        .post_json::<SavePropertyPayload, PropertyMutationResponse>(
            "/api/rpc/save_property",
            payload,
        )
        .await
}

async fn fallback_to_cache(
    pool: &SqlitePool,
    reason: &str,
) -> Result<PropertySummariesResult, AppError> {
    let cached = property_cache::read_summaries(pool).await?;

    if cached.rows.is_empty() {
        return Ok(PropertySummariesResult {
            rows: Vec::new(),
            source: PropertySource::Empty,
            last_synced_at: cached.last_synced_at,
            error: Some(reason.to_string()),
        });
    }

    Ok(PropertySummariesResult {
        rows: cached.rows,
        source: PropertySource::Cache,
        last_synced_at: cached.last_synced_at,
        error: Some(reason.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use super::*;
    use crate::{
        models::{PropertyKind, PropertyStatus},
        services::local_db,
    };

    fn spawn_property_server(payload: &'static str) -> std::net::SocketAddr {
        let payload = payload.as_bytes();
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = listener.local_addr().expect("test address should resolve");

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test request should connect");
            let mut buffer = [0u8; 4096];
            let read = stream.read(&mut buffer).unwrap_or(0);
            let request_line = std::str::from_utf8(&buffer[..read])
                .unwrap_or_default()
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            assert!(
                request_line.starts_with("GET /api/properties?"),
                "expected GET /api/properties?..., got {request_line:?}"
            );

            let mut response = Vec::new();
            response.extend_from_slice(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ",
            );
            response.extend_from_slice(payload.len().to_string().as_bytes());
            response.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
            response.extend_from_slice(payload);
            stream
                .write_all(&response)
                .expect("test response should write");
        });

        address
    }

    fn spawn_failing_server() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = listener.local_addr().expect("test address should resolve");

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test request should connect");
            let mut buffer = [0u8; 4096];
            let _ = stream.read(&mut buffer);
            stream
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
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

    #[tokio::test]
    async fn live_fetch_returns_rows_and_populates_cache() {
        let payload = r#"[
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "display_name": "Sample Sale",
                "kind": "sale",
                "status": "active",
                "address_raw": "address",
                "updated_at": "2026-05-10T12:34:56Z"
            }
        ]"#;

        let address = spawn_property_server(payload);
        let pool = local_db::open_in_memory().await;

        let result = list_property_summaries(&config_for(address), &pool)
            .await
            .unwrap();
        assert_eq!(result.source, PropertySource::Live);
        assert_eq!(result.rows.len(), 1);
        assert!(result.last_synced_at.is_some());
        assert!(result.error.is_none());

        let cached = property_cache::read_summaries(&pool).await.unwrap();
        assert_eq!(cached.rows.len(), 1);
        assert_eq!(cached.rows[0].kind, PropertyKind::Sale);
    }

    #[tokio::test]
    async fn server_failure_falls_back_to_cache_marked_stale() {
        let pool = local_db::open_in_memory().await;
        // Seed cache.
        property_cache::replace_summaries(
            &pool,
            &[PropertySummary {
                id: "cached-1".to_string(),
                display_name: "Cached Property".to_string(),
                kind: PropertyKind::Rental,
                status: PropertyStatus::Pending,
                address_raw: None,
                updated_at: None,
            }],
            Utc::now(),
        )
        .await
        .unwrap();

        let address = spawn_failing_server();
        let result = list_property_summaries(&config_for(address), &pool)
            .await
            .unwrap();

        assert_eq!(result.source, PropertySource::Cache);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].display_name, "Cached Property");
        assert!(result.error.is_some());
        assert!(result.last_synced_at.is_some());
    }

    #[tokio::test]
    async fn not_configured_with_empty_cache_returns_empty_source() {
        let pool = local_db::open_in_memory().await;
        let mut config = AppConfig::default();
        config.server.base_url = "   ".to_string();

        let result = list_property_summaries(&config, &pool).await.unwrap();
        assert_eq!(result.source, PropertySource::Empty);
        assert!(result.rows.is_empty());
        assert!(result.error.is_some());
        assert!(result.last_synced_at.is_none());
    }

    fn spawn_save_server(status_line: &'static str, body: &'static str) -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = listener.local_addr().expect("test address should resolve");

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test request should connect");
            let mut buffer = [0u8; 8192];
            let read = stream.read(&mut buffer).unwrap_or(0);
            let request = std::str::from_utf8(&buffer[..read]).unwrap_or_default();
            assert!(
                request.starts_with("POST /api/rpc/save_property"),
                "expected POST /api/rpc/save_property, got first line {:?}",
                request.lines().next().unwrap_or(""),
            );

            let mut response = Vec::new();
            response.extend_from_slice(status_line.as_bytes());
            response.extend_from_slice(b"\r\nContent-Type: application/json\r\nContent-Length: ");
            response.extend_from_slice(body.len().to_string().as_bytes());
            response.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
            response.extend_from_slice(body.as_bytes());
            stream
                .write_all(&response)
                .expect("test response should write");
        });

        address
    }

    fn sample_payload() -> SavePropertyPayload {
        SavePropertyPayload {
            id: None,
            display_name: "New Listing".to_string(),
            kind: PropertyKind::Sale,
            address_raw: Some("台北市".to_string()),
        }
    }

    #[tokio::test]
    async fn save_property_posts_and_parses_response() {
        let response_body = r#"{"id":"abc-123","updatedAt":"2026-05-18T08:00:00Z"}"#;
        let address = spawn_save_server("HTTP/1.1 200 OK", response_body);

        let response = save_property(&config_for(address), &sample_payload())
            .await
            .expect("save should succeed");

        assert_eq!(response.id, "abc-123");
    }

    #[tokio::test]
    async fn save_property_rejects_empty_display_name_before_network() {
        let config = config_for("127.0.0.1:1".parse().unwrap());
        let payload = SavePropertyPayload {
            display_name: "   ".to_string(),
            ..sample_payload()
        };

        let result = save_property(&config, &payload).await;
        assert!(matches!(result, Err(AppError::InvalidInput { .. })));
    }

    #[tokio::test]
    async fn save_property_refuses_when_server_unconfigured() {
        let mut config = AppConfig::default();
        config.server.base_url = "   ".to_string();

        let result = save_property(&config, &sample_payload()).await;
        assert!(matches!(result, Err(AppError::InvalidInput { .. })));
    }

    #[tokio::test]
    async fn save_property_surfaces_validation_error_from_server() {
        let address = spawn_save_server(
            "HTTP/1.1 400 Bad Request",
            r#"{"kind":"validation","message":"display_name is required"}"#,
        );

        let result = save_property(&config_for(address), &sample_payload()).await;
        let error = result.expect_err("400 should surface as error");
        let AppError::ServerUnreachable { message } = error else {
            panic!("expected ServerUnreachable, got {error:?}");
        };
        assert!(message.contains("400"));
        assert!(message.contains("display_name"));
    }
}
