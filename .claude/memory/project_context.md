---
name: Project context
description: Core architecture decisions — Rust+Tauri desktop, Docker on internal server, evidence-first AI, plugin boundaries
type: project
---

Realestate_Management_Apps is a greenfield Rust + Tauri desktop app replacing the legacy Owner-Property-Management-AI-SPA (Next.js). The legacy app remains untouched.

**Why:** The rebuild separates desktop UI from heavy backend services, adds evidence-first AI workflows, and establishes explicit plugin contracts between three apps (Realestate, SayDo, Project-Manager).

**How to apply:** Every implementation decision should check: (1) does this keep the desktop thin? (2) does AI output go through the evidence pipeline? (3) is the plugin boundary explicit and versioned?

Key facts:
- Internal server: rick@192.168.1.6 (Docker services)
- Company standards: /Volumes/KLEVV-4T-1/Company-AI-App-Standards/ v0.2
- Legacy reference: /Volumes/KLEVV-4T-1/Real Estate Management Projects/Owner-Property-Management-AI-SPA/
- Phase: Documentation/Architecture (Phase 0), heading into scaffold (Phase 1)
