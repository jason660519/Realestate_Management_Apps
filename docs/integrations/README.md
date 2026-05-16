# Integrations

This app follows Company AI App Standards v0.2 for cross-app plugin boundaries.

## Platform Peers

- SayDo
- Project-Manager
- Realestate_Management_Apps

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
