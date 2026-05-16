# Dev Log: Config Persistence

Date: 2026-05-17

## Completed

- Added a Rust config store that reads and writes `config.toml` under the Tauri app data directory.
- Changed `AppState` so app config loads from disk at startup and persists updates with a temporary-file replace.
- Kept frontend/Tauri JSON payloads camelCase while storing TOML keys in snake_case.
- Added Rust tests for config create/load, config patch persistence, plugin status mapping, and degraded/not-configured server health responses.
- Updated the settings save notification to reflect persisted local app data.

## Technical Notes

- Tauri app data path is resolved with `app.path().app_data_dir()` during app setup.
- The TOML persistence model is intentionally separate from the API model so frontend serialization remains stable.
- Health checks now trim whitespace before checking for an empty server URL.

## Next Priority

- Split the current `src/router.tsx` shell into route and component modules before the file grows further.
- Add UI controls for plugin enablement now that config patch persistence is durable.
- Start Phase 2 property/document data surfaces only after the desktop foundation remains green.
