# F01 TDD Spec - Frontend React Stack

## Summary

F01 testing centers on the frontend shell split and shared UI state. The existing dev log records route modularization, shell-level app data refresh, Settings plugin toggles, and Integrations state reuse.

## Test Focus

- Route modules stay separated from route tree construction.
- Shell app data refresh keeps service health and plugin status consistent.
- Settings saves server URL and plugin toggles through the persisted config command.
- Integrations reads plugin state from shell context and avoids stale local fetch state.

## Source Doc

Original source: `docs/project-process/dev-logs/dev-frontend-shell-split-2026-05-17.md`

