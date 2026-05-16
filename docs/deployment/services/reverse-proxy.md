# Service Spec: reverse-proxy

Status: Draft  
Related: `docs/deployment/internal-server-plan.md`

## Role

Single internal HTTP entrypoint for the desktop app. It exposes health and API routes while keeping data services off host ports.

## Container

| Field | Value |
|---|---|
| Preferred image | `caddy:2` |
| Alternate image | `traefik:v3` |
| Exposed host port | `8080` on internal network only |
| Network | `realestate_internal` |
| Restart policy | `unless-stopped` |

## Routes

| Route | Upstream |
|---|---|
| `/health` | Aggregate health handler or static JSON during Phase 1 |
| `/api/*` | `postgrest:3000` |
| `/storage/*` | object storage, Phase 2+ |
| `/ai/*` | AI broker, Phase 3+ |

## Security

- Bind only to the internal network.
- Do not expose the app publicly.
- Keep Postgres unexposed.
- TLS can use an internal self-signed certificate after basic health checks work.

## Verification

```bash
curl http://192.168.1.6:8080/health
curl http://192.168.1.6:8080/api/
```
