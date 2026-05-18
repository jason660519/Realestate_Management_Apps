use tauri::State;

use crate::{
    errors::AppError, models::PropertySummary, services::property_service, state::AppState,
};

/// Read-only batch of property summaries for the desktop list view.
///
/// v0 surface: hits PostgREST `/api/properties` through the shared `ServerClient`.
/// Returns an empty list when the server URL is unconfigured so the UI can surface
/// a `not configured` state without an error toast.
#[tauri::command]
pub async fn list_property_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<PropertySummary>, AppError> {
    let config = state.config()?;
    property_service::list_property_summaries(&config).await
}
