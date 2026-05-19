# F02 Feature Spec - Backend API Development

## Summary

F02 defines the backend/API boundary for exporting real estate workflow issues to Project-Manager. The contract is versioned, permission-gated, and avoids direct access to Realestate_Management_Apps local state, files, databases, secrets, or Docker volumes.

## Canonical Scope

- Provide a `realestate.task.export` capability at contract version `0.1.0`.
- Export task title, type, priority, source entity references, evidence summary, requested action, and callback correlation id.
- Receive Project-Manager status updates through a matching callback payload.
- Preserve local pending state and show degraded sync when Project-Manager is offline or rejects payloads.

## Source Doc

Original source: `docs/integrations/project-manager-task-contract-v0.md`

