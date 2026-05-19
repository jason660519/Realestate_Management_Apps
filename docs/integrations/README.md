# Integrations

This app follows Company AI App Standards v0.2 for cross-app plugin boundaries.

## Platform Peers

- SayDo
- Project-Manager
- Realestate_Management_Apps
- OpenClaw (AI agent sidecar)

## OpenClaw Bridge

The `@jason66/shared-bridge` package (`/Volumes/KLEVV-4T-1/shared-bridge/`) provides shared types and an `OpenClawBridge` client class. OpenClaw runs as a sidecar in Project-Manager and can relay events, report health, and dispatch agent tasks across apps through its gateway at `http://127.0.0.1:18790`.

| Capability | Direction | Transport |
|---|---|---|
| `realestate.task.export` | Realestate → PM | HTTP API |
| `saydo.text.handoff` | SayDo → Realestate | Local IPC |

## Initial Direction

Realestate_Management_Apps should expose property, document, task, and AI-review capabilities through explicit plugin contracts. It should not share private local state or database tables directly with SayDo or Project-Manager.

## Contract Drafts

- `project-manager-task-contract-v0.md` — Realestate_Management_Apps exports taskable issues to Project-Manager.
- `saydo-input-contract-v0.md` — SayDo hands off operator-approved text payloads to Realestate_Management_Apps.

## Required For Each Plugin

- Provider and consumer app.
- Capability name and version.
- Input and output schema.
- Permission scope.
- Error and degraded-mode behavior.
- Verification path.
