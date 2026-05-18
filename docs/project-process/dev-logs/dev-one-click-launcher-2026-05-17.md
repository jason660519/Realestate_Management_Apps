# One-Click Launcher Dev Log

Date: 2026-05-17

## Completed

- Added `dev-start.command` at the repo root for macOS double-click development startup.
- The launcher starts the Tauri desktop app with `npm run tauri:dev`.
- The launcher opens the Vite web page at `http://127.0.0.1:5173` once the dev server responds.
- The launcher checks for existing port 5173 listeners before startup to reduce wrong-project or stale-server confusion.
- Updated `README.md` with the one-click startup path.

## Technical Notes

- Tauri already runs `npm run dev` through `beforeDevCommand`, so the launcher delegates to `npm run tauri:dev` instead of starting a separate Vite process.
- The browser-open helper waits for the web URL while the Tauri command stays in the foreground.
- The launcher does not hardcode any backend server IP or service URL.

## Next Priority

- Build a production `.app` or `.dmg` once the app icon and signing configuration are ready.
