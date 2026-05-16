# Deployment

This repo targets a desktop app with service-backed Docker containers on the home internal server.

## Internal Server

Current target server:

```text
rick@192.168.1.6
```

Keep this as deployment configuration. Do not hard-code the address in source modules.

## Required Service Docs

For each service, document:

- Service name.
- Environment variable or local config key.
- Port and protocol.
- Health check.
- Data volume boundary.
- Backup and restore note.
