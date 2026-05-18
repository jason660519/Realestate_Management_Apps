use crate::{
    errors::AppError,
    models::{AppConfig, PropertySummary},
    services::server_client::ServerClient,
};

/// PostgREST `select` projection for the property summary surface. Keep the column
/// list in sync with `PropertySummary` — adding or renaming a field requires updating
/// both this constant and the model.
const SUMMARY_SELECT: &str = "id,display_name,kind,status,address_raw,updated_at";
const SUMMARY_DEFAULT_LIMIT: u32 = 100;

/// Fetch a read-only batch of property summaries from the server.
///
/// Returns `Ok(vec![])` when the server URL is blank — UI surfaces should render the
/// `not configured` state rather than treat empty config as a transient failure.
pub async fn list_property_summaries(config: &AppConfig) -> Result<Vec<PropertySummary>, AppError> {
    let Some(client) = ServerClient::from_config(config)? else {
        return Ok(Vec::new());
    };

    let path = format!(
        "/api/properties?select={SUMMARY_SELECT}&order=updated_at.desc&limit={SUMMARY_DEFAULT_LIMIT}"
    );

    client.get_json::<Vec<PropertySummary>>(&path).await
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use super::*;
    use crate::models::{PropertyKind, PropertyStatus};

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

    fn config_for(address: std::net::SocketAddr) -> AppConfig {
        let mut config = AppConfig::default();
        config.server.base_url = format!("http://{address}");
        config.server.timeout_sec = 2;
        config
    }

    #[test]
    fn list_property_summaries_parses_postgrest_response() {
        let payload = r#"[
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "display_name": "Sample Sale",
                "kind": "sale",
                "status": "active",
                "address_raw": "台北市信義區松仁路 1 號",
                "updated_at": "2026-05-10T12:34:56Z"
            },
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "display_name": "Sample Rental",
                "kind": "rental",
                "status": "draft",
                "address_raw": null,
                "updated_at": null
            }
        ]"#;

        let address = spawn_property_server(payload);
        let summaries =
            tauri::async_runtime::block_on(list_property_summaries(&config_for(address)))
                .expect("summaries should load");

        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].kind, PropertyKind::Sale);
        assert_eq!(summaries[0].status, PropertyStatus::Active);
        assert_eq!(
            summaries[0].address_raw.as_deref(),
            Some("台北市信義區松仁路 1 號"),
        );
        assert_eq!(summaries[1].kind, PropertyKind::Rental);
        assert_eq!(summaries[1].status, PropertyStatus::Draft);
        assert!(summaries[1].address_raw.is_none());
    }

    #[test]
    fn list_property_summaries_returns_empty_when_server_not_configured() {
        let mut config = AppConfig::default();
        config.server.base_url = "   ".to_string();

        let summaries = tauri::async_runtime::block_on(list_property_summaries(&config))
            .expect("not_configured should not error");
        assert!(summaries.is_empty());
    }

    #[test]
    fn unknown_property_kind_does_not_break_parsing() {
        let payload = r#"[
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "display_name": "Legacy Mixed-Use",
                "kind": "mixed_use",
                "status": "pending",
                "address_raw": null,
                "updated_at": null
            }
        ]"#;

        let address = spawn_property_server(payload);
        let summaries =
            tauri::async_runtime::block_on(list_property_summaries(&config_for(address)))
                .expect("summaries should load");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].kind, PropertyKind::Unknown);
        assert_eq!(summaries[0].status, PropertyStatus::Pending);
    }
}
