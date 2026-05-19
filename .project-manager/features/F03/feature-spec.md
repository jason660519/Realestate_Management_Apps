# F03 Feature Spec - Database Integration

## Summary

F03 establishes the canonical database direction for the desktop rebuild. The app uses a lean self-hosted Supabase-compatible stack on the internal server: Postgres 17 as canonical data storage and PostgREST v12 for read-oriented REST access, with writes routed through Rust/service-role paths.

## Canonical Scope

- Keep canonical data on the internal server rather than local-only SQLite.
- Use Postgres plus PostgREST as the MVP data stack.
- Keep anon access read-only and route writes through Rust server-side service-role paths.
- Preserve evidence-first schema conventions for source-backed fields.
- Keep local storage for app config, cache, queue metadata, and diagnostics.

## Source Doc

Original source: `docs/architecture/ADR-006-database-supabase-self-hosted.md`

