# SayDo Input Contract v0

Status: Draft v0  
Date: 2026-05-17  
Provider: SayDo  
Consumer: Realestate_Management_Apps  
Related: `docs/product/prd.md` §7.1, Company AI App Standards v0.2

## Purpose

SayDo can hand off operator-approved text to Realestate_Management_Apps for property notes, showing summaries, customer requirements, or document intake context. The handoff is text-first and permissioned. Raw audio and SayDo local state remain inside SayDo unless a future contract explicitly grants access.

## Capability

| Field | Value |
|---|---|
| Capability name | `saydo.text.handoff` |
| Version | `0.1.0` |
| Direction | SayDo -> Realestate_Management_Apps |
| Initial transport | Local IPC, HTTP callback, or file handoff; final transport selected when SayDo integration starts |
| Persistence owner | SayDo owns recording/transcript source; Realestate_Management_Apps owns imported text draft after operator acceptance |

## Permission Scope

Allowed:
- Import a text payload approved by the operator.
- Attach the text to a property draft, document intake note, AI review note, or customer requirement draft.
- Store a source reference to the SayDo handoff id.

Not allowed:
- Read SayDo local database.
- Read raw audio without a separate raw-media permission.
- Use SayDo provider/model settings as Realestate_Management_Apps settings.
- Auto-save high-risk property facts as canonical data.

## Input Payload

```json
{
  "contractVersion": "0.1.0",
  "sourceApp": "SayDo",
  "handoffId": "uuid",
  "createdAt": "2026-05-17T00:00:00Z",
  "language": "zh-Hant-TW",
  "contentKind": "showing_note",
  "text": "客戶偏好三房兩廳，需近捷運，預算約 2500 萬。",
  "source": {
    "saydoSessionId": "uuid",
    "transcriptId": "uuid",
    "rawAudioAvailable": false
  },
  "routingHint": {
    "targetSurface": "property_draft",
    "propertyId": "uuid"
  },
  "operatorApproval": {
    "approvedBy": "local_operator",
    "approvedAt": "2026-05-17T00:00:00Z"
  }
}
```

## Output Payload

```json
{
  "contractVersion": "0.1.0",
  "handoffId": "uuid",
  "status": "accepted",
  "importedDraftId": "uuid",
  "message": "Text handoff stored as a review draft.",
  "receivedAt": "2026-05-17T00:00:00Z"
}
```

## Enumerations

`contentKind`:
- `property_note`
- `showing_note`
- `customer_requirement`
- `document_context`
- `task_note`
- `other`

`targetSurface`:
- `workbench`
- `property_draft`
- `document_intake`
- `ai_review`
- `task_draft`

`status`:
- `accepted`
- `rejected`
- `needs_operator_review`
- `contract_version_unsupported`

## Evidence and Canonical Save Rules

SayDo text is imported as draft context only. If the text contains high-risk facts such as ownership, area, price, legal, or financial details, Realestate_Management_Apps must route those values through the evidence review workflow before canonical save.

No imported SayDo text can silently overwrite confirmed property fields.

## Error and Degraded Mode

If Realestate_Management_Apps is offline, SayDo should keep the handoff pending or export a file packet for manual import.

If Realestate_Management_Apps cannot match the requested property id, it accepts the text only as an unattached draft or rejects with `target_not_found`.

If the operator approval block is missing, the handoff is stored as `needs_operator_review`.

## Verification

Phase 1 manual verification:

1. Enable SayDo placeholder in Integrations.
2. Paste or load a sample handoff payload.
3. Confirm Realestate_Management_Apps displays the handoff as a draft context, not canonical property data.

Phase 4 integration verification will add SayDo-side contract tests.
