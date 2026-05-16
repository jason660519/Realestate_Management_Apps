# Service Spec: postgrest

Status: Draft  
Related: `docs/architecture/ADR-006-database-supabase-self-hosted.md`

## Role

PostgREST exposes the initial `/api/*` REST surface backed by Postgres. It is used to move Phase 2 property/document CRUD forward before a custom Rust API is needed.

## Container

| Field | Value |
|---|---|
| Image | `postgrest/postgrest:v12` |
| Internal port | `3000` |
| Host exposure | None |
| Reverse proxy route | `/api/*` |
| Network | `realestate_internal` |
| Restart policy | `unless-stopped` |

## Environment

| Variable | Source |
|---|---|
| `PGRST_DB_URI` | `.env.local`, points to `postgres:5432` |
| `PGRST_DB_SCHEMAS` | `public` |
| `PGRST_DB_ANON_ROLE` | `anon` |
| `PGRST_JWT_SECRET` | `.env.local` |
| `PGRST_SERVER_PORT` | `3000` |

## Permission Posture

v0 starts with `anon` read-only. Writes should go through a future Rust service path holding service role credentials, not direct WebView requests.

## Health Check

PostgREST root should return schema metadata through the reverse proxy:

```bash
curl http://192.168.1.6:8080/api/
```

## Verification

```bash
docker compose ps postgrest
curl http://192.168.1.6:8080/api/
```
