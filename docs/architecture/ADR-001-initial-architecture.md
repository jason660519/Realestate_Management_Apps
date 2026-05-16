# ADR-001: Initial Architecture

Status: Proposed  
Date: 2026-05-17  
Owner: Project lead

## Context

The legacy real estate management app should remain unchanged while this repo becomes the new Rust + Tauri desktop implementation. Shared Docker services will run on the home internal server, and the app must eventually plug into SayDo and Project-Manager.

## Decision

Use the company baseline documentation, design, multi-app integration, and AI engineer workflow structure. Treat Rust + Tauri as the target app shell, with explicit plugin and service boundaries.

## Consequences

The first implementation phase should scaffold the desktop app and runtime contracts before migrating legacy features. The legacy app remains a source of product behavior, data model knowledge, and migration evidence.

## Alternatives Considered

Continuing inside the legacy Next.js SPA was rejected because the user wants a separate desktop app repo and no disruption to the old solution.

## Verification

Run:

```bash
npm run standards:check
```
