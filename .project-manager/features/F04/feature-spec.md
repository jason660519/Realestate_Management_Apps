# F04 Feature Spec - DevOps Pipeline

## Summary

F04 defines the deployment and operations plan for the internal server that backs the desktop app. The server should expose one controlled internal HTTP entry point, run services in separated Docker networks/volumes, provide visible health checks, and keep backup/restore expectations documented.

## Canonical Scope

- Use a reverse proxy as the single internal HTTP entry point.
- Run MVP data services as Postgres 17 plus PostgREST v12.
- Add service specs for proxy, data, storage, AI, and observability as they enter scope.
- Keep secrets out of source and `.env.local` out of git.
- Provide aggregate health status and backup/restore drill expectations.

## Source Doc

Original source: `docs/deployment/internal-server-plan.md`

