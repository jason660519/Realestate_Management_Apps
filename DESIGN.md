# Realestate_Management_Apps Design Guide

Status: Repo-specific implementation guide  
Company standard: Company AI App Standards v0.2

## Read This First

1. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/ui-design-system.md`
2. `DESIGN.md`
3. `README.md`
4. `docs/architecture/README.md`
5. `docs/product/prd.md`

## Product Personality

This app is an operational real estate management desktop tool. It should feel calm, dense, trustworthy, and built for repeated professional use, not like a marketing website.

## Shell Pattern

Use a desktop app shell with persistent navigation, compact work surfaces, status visibility, and drill-down panels. Prioritize property operations, document intake, AI review, GIS/document evidence, task coordination, and plugin status.

## Information Architecture

Top-level navigation should follow user jobs:

- Workbench
- Properties
- Documents
- AI Review
- Tasks
- Integrations
- Settings

## Project-Specific Overrides

The real estate domain requires evidence-first UI. AI-generated or AI-extracted content must show source, confidence, failed model/provider state, and human confirmation before canonical save when legal or property data is affected.

## Acceptance Checklist

- Company design system was followed.
- Repo-specific overrides were checked.
- Build/typecheck pass when implementation exists.
- Text fits at desktop and narrow widths.
- Empty, loading, error, disabled, and blocked states are handled.
- Risky or externally visible actions show scope and consequence.
