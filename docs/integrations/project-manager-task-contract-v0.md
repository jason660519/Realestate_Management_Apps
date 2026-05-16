# Project-Manager Task Contract v0

Status: Draft v0  
Date: 2026-05-17  
Provider: Realestate_Management_Apps  
Consumer: Project-Manager  
Related: `docs/product/prd.md` §7.2, Company AI App Standards v0.2

## Purpose

Realestate_Management_Apps can create taskable issues when property, document, GIS, or AI review workflows need human follow-up. Project-Manager receives a narrow task payload and returns task status updates through the same contract boundary.

This contract does not permit Project-Manager to read Realestate_Management_Apps local state, database tables, files, secrets, or Docker volumes.

## Capability

| Field | Value |
|---|---|
| Capability name | `realestate.task.export` |
| Version | `0.1.0` |
| Direction | Realestate_Management_Apps -> Project-Manager |
| Initial transport | HTTP API or file handoff, to be selected during Project-Manager integration |
| Persistence owner | Realestate_Management_Apps owns source issue and pending sync state; Project-Manager owns task workflow state |

## Permission Scope

The operator must explicitly enable Project-Manager integration before exporting tasks.

Allowed:
- Export task title, type, priority, source entity references, evidence summary, and callback correlation id.
- Receive status updates for exported task ids.

Not allowed:
- Direct database access.
- Raw document download unless a future permission adds document packet export.
- Secret sharing.
- Silent task creation when Project-Manager is unreachable.

## Export Payload

```json
{
  "contractVersion": "0.1.0",
  "sourceApp": "Realestate_Management_Apps",
  "sourceIssueId": "uuid",
  "correlationId": "uuid",
  "taskType": "document_missing_data",
  "priority": "normal",
  "title": "補齊謄本權利範圍確認",
  "description": "AI review requires operator confirmation before canonical save.",
  "sourceEntity": {
    "kind": "property",
    "id": "uuid",
    "displayName": "string"
  },
  "evidence": {
    "documentId": "uuid",
    "processingRunId": "uuid",
    "stageName": "review",
    "fieldPaths": ["ownership.owners", "landArea"],
    "summary": "Human confirmation required; no canonical write has occurred."
  },
  "requestedAction": "review_and_confirm",
  "createdAt": "2026-05-17T00:00:00Z"
}
```

## Status Callback Payload

```json
{
  "contractVersion": "0.1.0",
  "projectManagerTaskId": "uuid",
  "correlationId": "uuid",
  "status": "completed",
  "resolution": "confirmed",
  "note": "Operator confirmed ownership fields.",
  "updatedAt": "2026-05-17T00:00:00Z"
}
```

## Enumerations

`taskType`:
- `document_missing_data`
- `ai_review_failed`
- `human_confirmation_required`
- `gis_manual_fallback_required`
- `property_data_conflict`

`priority`:
- `low`
- `normal`
- `high`
- `blocking`

`status`:
- `accepted`
- `in_progress`
- `blocked`
- `completed`
- `rejected`

## Error and Degraded Mode

If Project-Manager is offline, Realestate_Management_Apps stores a local pending export record and shows the sync as degraded. It must not mark a task as exported until Project-Manager acknowledges it.

If Project-Manager rejects a payload, Realestate_Management_Apps keeps the source issue open and stores the rejection reason.

If contract versions do not match, both apps must fail visibly with `contract_version_unsupported`.

## Verification

Phase 1 manual verification:

1. Enable Project-Manager placeholder in Integrations.
2. Export a sample task payload from the Tasks surface.
3. Confirm the pending state remains local when no Project-Manager endpoint is configured.

Phase 4 integration verification will add a real Project-Manager endpoint test.
