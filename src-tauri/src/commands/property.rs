use tauri::State;

use crate::{
    errors::AppError,
    models::{PropertyMutationResponse, PropertySummariesResult, SavePropertyPayload},
    services::property_service,
    state::AppState,
};

/// Read-only batch of property summaries for the desktop list view.
///
/// Backed by the shared `ServerClient` for the live fetch and the SQLite cache
/// for offline fall-back. The wrapper carries `source` (`live` / `cache` /
/// `empty`) and `lastSyncedAt` so the UI can render a stale banner without
/// re-asking other surfaces.
#[tauri::command]
pub async fn list_property_summaries(
    state: State<'_, AppState>,
) -> Result<PropertySummariesResult, AppError> {
    let config = state.config()?;
    let pool = state.local_db();
    property_service::list_property_summaries(&config, pool).await
}

/// Skeleton write command for the axum service path (ADR-010).
///
/// v0 only carries the minimal payload needed for the wire — full
/// evidence-backed semantics arrive once the axum service contract lands.
/// The desktop never falls back to a cache-only path on save: saves must
/// reach the canonical server or fail loudly.
#[tauri::command]
pub async fn save_property(
    state: State<'_, AppState>,
    payload: SavePropertyPayload,
) -> Result<PropertyMutationResponse, AppError> {
    let config = state.config()?;
    property_service::save_property(&config, &payload).await
}
