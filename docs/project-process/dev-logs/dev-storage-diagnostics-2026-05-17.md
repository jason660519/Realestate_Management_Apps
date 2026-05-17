# Dev Log: Storage Diagnostics

Date: 2026-05-17

## Completed

- Added a Tauri command for local storage diagnostics.
- Exposed app data directory, `config.toml` path, existence, readability, file size, and filesystem error state to the frontend.
- Added a Settings page diagnostics panel so persisted config location is visible to operators.
- Added preview-mode fallback diagnostics for frontend-only Vite runs.
- Added a Rust unit test for created config diagnostics.

## Technical Notes

- Diagnostics are read-only and do not write probe files.
- The command uses the existing `ConfigStore`, keeping config path ownership in one service.
- Long config paths wrap with `.path-text` to avoid overflowing the Settings panel.

## Next Priority

- Add app icon replacement for the Tauri bundle.
- Consider a narrow Settings smoke test once the frontend test harness is established.
- Keep Phase 2 property/document work behind explicit server/data boundary decisions.
