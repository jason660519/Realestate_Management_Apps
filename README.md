# Realestate_Management_Apps

Rust + Tauri desktop rebuild of the real estate management product.

This repo is the new app. The legacy `Owner-Property-Management-AI-SPA` remains unchanged and is treated as a reference and migration source, not as the implementation target.

## Product Docs

- `docs/product/prd.md`
- `docs/architecture/README.md`
- `docs/integrations/README.md`
- `docs/deployment/README.md`

## Development

On macOS, double-click `dev-start.command` to start the Tauri desktop app and open the Vite web page at `http://127.0.0.1:5173`.

```bash
npm install
npm run standards:doctor
npm run dev
npm run build
npm run tauri:dev
```

Current Phase 1 scaffold includes a Vite + React WebView, Tauri 2 shell, settings command skeleton, server health check, and plugin registry placeholder.

Verification:

```bash
npm run standards:check
npm run typecheck
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```
