# F03 TDD Spec - Database Integration

## Summary

F03 testing covers local storage diagnostics and config-path visibility. The existing dev log records a Tauri diagnostic command, frontend Settings diagnostics panel, preview-mode fallback diagnostics, and Rust unit coverage for created config diagnostics.

## Test Focus

- Diagnostics are read-only and do not write probe files.
- Config path ownership stays in `ConfigStore`.
- Settings displays app data directory, config path, file existence, readability, size, and filesystem error state.
- Long paths wrap without overflowing the Settings panel.

## Source Doc

Original source: `docs/project-process/dev-logs/dev-storage-diagnostics-2026-05-17.md`

