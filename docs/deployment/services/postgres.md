# Service Spec: postgres

Status: Draft  
Related: `docs/architecture/ADR-006-database-supabase-self-hosted.md`

## Role

Canonical database for Realestate_Management_Apps.

## Container

| Field | Value |
|---|---|
| Image | `postgres:17` |
| Internal port | `5432` |
| Host exposure | None |
| Network | `realestate_internal` |
| Restart policy | `unless-stopped` |

## Environment

| Variable | Source |
|---|---|
| `POSTGRES_DB` | `.env.local` |
| `POSTGRES_USER` | `.env.local` |
| `POSTGRES_PASSWORD` | `.env.local` |

Secrets must not be committed.

## Extensions

Install during bootstrap:

- `uuid-ossp`
- `pgcrypto`
- `postgis`

PostGIS is installed for future GIS workflows but v1 data model does not use geometry/geography columns yet.

## Volumes

| Volume | Purpose | Backup |
|---|---|---|
| `pg_data` | Canonical Postgres data | Daily `pg_dump`, encrypted backup |

## Health Check

```bash
pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"
```

## Verification

```bash
docker compose ps postgres
docker compose exec postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"
```
