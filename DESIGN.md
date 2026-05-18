# Realestate_Management_Apps Design Guide

Status: Source of truth for Realestate Management UI implementation
Company standard: Company AI App Standards v0.2
Audience: AI engineers, frontend engineers, product/design collaborators
Last updated: 2026-05-17

## Read This First

Before changing Realestate Management UI, read these files in order:

1. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/ui-design-system.md` - company baseline design system.
2. `DESIGN.md` - this repo-specific implementation guide.
3. `docs/design/shared-ai-desktop-style.md` - shared visual language for the company AI desktop app family.
4. `README.md` - app purpose and current scope.
5. `docs/architecture/README.md` - architecture and ADR index.
6. `docs/product/prd.md` - product requirements.

## Product Personality

This app is an operational real estate management desktop tool. It should feel calm, dense, trustworthy, and built for repeated professional use. It is not a marketing website or a generic admin panel.

Core identity:
- Evidence-first: AI-generated content must show source, confidence, and human confirmation gates.
- Operational density: Dense tables, split panels, compact forms.
- Trustworthy: State, fallback, permission, and execution risk are always visible.

## Current Shell Pattern

Current implementation files:
- `src/components/shell/ShellLayout.tsx`
- `src/components/shell/navigation.ts`
- `src/components/shell/StatusPanel.tsx`
- `src/components/shell/PluginPanel.tsx`
- `src/styles.css`

Realestate Management uses the company-standard desktop app shell:

```text
RE icon rail | sticky topbar + scrollable workbench + optional right panels
```

Rules:
- Keep the `RE` rail mark visible.
- Rail icon navigation only; no second text sidebar column.
- Navigation items must be job-based with recognizable icons.
- Active nav state uses a clear border and subtle success background.
- Use the sticky topbar for page title, current mode, and health badge.
- Right panels (service health, plugin boundary) appear at wide screens only.
- Main content should be dense operational UI, not decorative marketing.

## Information Architecture

Top-level navigation follows user jobs:

| Area | Purpose |
|---|---|
| Workbench | Operational overview and document evidence stage visualization |
| Properties | Browse, search, and manage property records |
| Documents | Intake, parse, organize, and archive property documents |
| AI Review | Review AI-extracted content with source, confidence, and approval gates |
| Tasks | Coordinate and track property/document work items |
| Integrations | Plugin contracts for SayDo, Project-Manager, and backend services |
| Settings | Local app config, server URL, plugin toggles, storage diagnostics |

Do not add new top-level nav items for implementation modules. Prefer nesting under existing areas.

## Visual Tokens

Use `docs/design/shared-ai-desktop-style.md` as the shared token source. Realestate Management adopts the company baseline:

| Token | Value | Use |
|---|---|---:|
| App background | `#071b18` | Root app background |
| Rail background | `#061512` | Left icon rail |
| Panel background | `rgba(255, 255, 255, 0.05)` | Main content panels |
| Panel border | `rgba(231, 229, 228, 0.13)` | Section and panel borders |
| Strong text | `#f5f5f4` | Headings and primary labels |
| Muted text | `rgba(214, 211, 209, 0.75)` | Secondary metadata |
| Amber accent | `#fef3c7` | App mark `RE`, key highlights |
| Active blue | `#2563eb` | Primary CTA and active step |
| Success | `#14532d` | Ready, connected, granted |
| Danger | `#7f1d1d` | Failed, missing, destructive |

Real estate domain additions (document by ADR if needed):
- `#fef3c7` (amber) for evidence flags and review-required badges.
- `rgba(209, 250, 229, 0.9)` (emerald) for the rail status indicator.

## Component Standards

### Navigation
- Use `@tabler/icons-react` icons for rail navigation (consistent with Mantine ecosystem).
- App mark is `RE`, not a generic icon.
- Rail items include `title` attributes for tooltip clarity.

### Tables
Use tables for property lists, document indexes, task queues, and review queues.

Table requirements:
- Stable status column.
- Compact row height.
- Clear empty state.
- Loading and error states.
- No hidden destructive actions.

### Cards
Cards are allowed for repeated independent entities such as plugins or metrics.

Rules:
- Do not nest cards.
- Do not use cards as page sections.
- Use card grids sparingly; prefer tables for sortable data.

### Forms
- Use Mantine form components with compact sizing.
- Separate basic settings from advanced/server internals.
- Show saved/missing/degraded states visibly.
- Error messages must explain what failed, what was preserved, and next action.

### Secrets and Server Configuration
- Server URLs and configuration stored through Tauri Rust commands.
- Never log or render raw secrets in the frontend.

## Evidence-First UI Rules

The real estate domain mandates evidence-first UI. When showing AI-generated or AI-extracted content:

- Every extract must show source document, confidence score, and extraction timestamp.
- Failed model or provider state must be visible, not silently hidden.
- Human confirmation is required before canonical save when legal or property data is affected.
- "Evidence chain" stages (Detect -> Parse -> Review -> Human confirm -> Save) must remain visually distinct.
- Never simulate success when a stage failed or used degraded fallback.

## Plugin and Integration UX

Plugin panels and integration views must show:
- Source app identity (SayDo, Project-Manager).
- Requested permission scope.
- Execution target and transport.
- Last sync/handshake status.
- Error state with recovery action.

Do not hardcode deployment IPs in shared modules. Server URL is runtime config.

## Copy Rules

Default UI copy should be short and operational.

Good:
- `Audio saved locally. Transcription failed. Retry when online.`
- `Evidence review required before save.`
- `Server health: degraded – 2 of 5 services unreachable.`

Avoid:
- `Something went wrong`
- `AI failed`
- `Done` when fallback or degraded output occurred.

Targets English and Traditional Chinese UI. Technical identifiers remain English.

## AI Engineer Rules

When implementing UI:
1. Check `src/components/shell/ShellLayout.tsx` and `src/styles.css` for existing patterns first.
2. Reuse Mantine components already configured in `src/theme.ts`.
3. Prefer `@tabler/icons-react` for icons.
4. Do not create a new color palette or decorative hero sections.
5. Keep normal users away from provider/server internals unless explicitly needed.
6. Include failure, empty, loading, disabled, and blocked states for every flow.
7. Verify with `npm run typecheck` and `npm run build` before handing off.

## Acceptance Checklist

Before calling a Realestate Management UI change done:
- Company design system and shared style guide were followed.
- Root `DESIGN.md` rules were followed.
- `npm run typecheck` passes.
- `npm run build` passes.
- Shell and navigation remain intact.
- Evidence-first rules are respected for any AI-content display.
- Empty/error/loading/disabled states are handled.
- Text fits at desktop and narrow widths without overflow.
- No raw secret or server credential is rendered in frontend state.
