# Architecture

This folder contains architecture decision records for the Rust + Tauri rebuild.

## ADRs

- `ADR-001-initial-architecture.md` — Initial direction: Rust + Tauri rebuild, legacy as reference
- `ADR-002-tauri-service-architecture.md` — Three-layer split: WebView / Rust command surface / internal server
- `ADR-003-tauri-2-adoption.md` — Adopt Tauri 2.x（capability model + plugin v2 + mobile option）
- `ADR-004-local-storage-and-secret.md` — Config TOML / SQLite state / fs cache / OS keychain secret
- `ADR-005-frontend-react-stack.md` — Frontend: React + TS + Mantine + TanStack + tauri-specta
- `ADR-006-database-supabase-self-hosted.md` — Database: Supabase self-hosted lean stack (postgres + PostgREST)

## Data Model

- `data-model-v1.md` — **Current**: aligns with legacy audit, Postgres DDL examples included
- `data-model-v0.md` — Superseded (kept as snapshot)

## Decision Inputs

- `frontend-and-db-trade-offs.md` — Source analysis behind ADR-005 and ADR-006

## Pending

- ADR-007: Auth / multi-user (when desktop expands beyond single operator)
- ADR-008: PostGIS adoption (when GIS workflow enters scope, Phase 3)
- i18n ADR (when AU / multi-region returns to scope; see `property_au_details` defer)

## Related Outside This Folder

- `docs/product/prd.md` — Product requirements
- `docs/deployment/internal-server-plan.md` — Server deployment skeleton
- `docs/migration/legacy-schema-audit.md` — 511-line legacy schema audit; primary input to data-model-v1
- `.claude/rules/backend/rust-tauri.md` — Rust/Tauri 開發慣例（dev rules）
