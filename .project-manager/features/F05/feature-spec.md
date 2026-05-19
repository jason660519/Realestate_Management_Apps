# F05 Feature Spec - Tauri Desktop App

## Summary

F05 establishes the Tauri desktop architecture. The app is split into a WebView UI layer, a Rust command surface for privileged operations, and an internal HTTP server for canonical data and heavy processing.

## Canonical Scope

- Keep WebView code focused on rendering, validation, and local component state.
- Route filesystem, secret, server, property, document, plugin, and diagnostics operations through Rust commands.
- Keep HTTP, filesystem, shell, dialog, notification, and clipboard capabilities explicitly scoped.
- Use visible error states for server, IO, permission, input, and plugin failures.
- Avoid returning secret values to the WebView.

## Source Doc

Original source: `docs/architecture/ADR-002-tauri-service-architecture.md`

