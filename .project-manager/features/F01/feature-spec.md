# F01 Feature Spec - Frontend React Stack

## Summary

F01 establishes the frontend foundation for the Tauri desktop app. The stack uses React with TypeScript strict mode, Vite, Mantine, TanStack Router, TanStack Query, TanStack Table, React Hook Form with zod, and Testing Library/Vitest. The feature exists to support dense, evidence-first real estate workflows inside a local desktop WebView.

## Canonical Scope

- Keep `src/router.tsx` focused on route tree construction.
- Use route modules and shared shell/page components for operational UI.
- Reuse Mantine, TanStack, and repo design-system patterns before adding new abstractions.
- Maintain evidence-first UI expectations: visible stage state, source, confidence, and human confirmation gates where AI output is involved.

## Source Doc

Original source: `docs/architecture/ADR-005-frontend-react-stack.md`

