# Dev Log: Frontend Shell Split

Date: 2026-05-17

## Completed

- Split the 520-line `src/router.tsx` into route modules, shell components, shared page components, and status utilities.
- Kept `src/router.tsx` focused on TanStack Router route tree construction.
- Added shell-level app data context so service health and plugin status share one refresh path.
- Wired Settings plugin toggles for SayDo and Project-Manager to the existing persisted config patch command.
- Updated the Integrations page to read plugin state from the shell context instead of owning a separate stale fetch.

## Technical Notes

- `src/components/shell/appData.tsx` owns health/plugin refresh state and wraps the shell outlet.
- Settings saves `serverBaseUrl`, `saydoEnabled`, and `projectManagerEnabled` together, then refreshes shell state.
- The plugin toggles remain permission placeholders; no cross-app transport is implemented in this step.

## Next Priority

- Add focused UI tests or interaction smoke coverage once the frontend test harness is established.
- Continue Phase 1 hardening with app icon replacement or config-path diagnostics.
- Start Phase 2 only after property/document data boundaries are explicit.
