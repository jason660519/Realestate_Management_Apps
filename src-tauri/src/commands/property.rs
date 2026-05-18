use tauri::State;

use crate::{
    errors::AppError, models::PropertySummariesResult, services::property_service, state::AppState,
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
