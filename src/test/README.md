# Frontend Test Harness

## Stack

- **Test runner**: `vitest` (jsdom environment).
- **DOM assertions**: `@testing-library/react`, `@testing-library/jest-dom`.
- **User interaction**: `@testing-library/user-event`.

Config lives in `vite.config.ts` under the `test:` block. Setup at `src/test/setup.ts` registers
jest-dom matchers and runs `cleanup()` after each test.

## Conventions

- Co-locate tests next to source: `foo.ts` → `foo.test.ts`.
- Pure utility tests (no React, no Tauri) are preferred whenever possible — they catch regressions
  cheaply and never need mocks.
- React component tests must wrap with the project Mantine theme (`src/theme.ts`). A reusable
  `renderWithMantine` helper will land in `src/test/render.tsx` when the first component test is
  added.
- Tauri `invoke` calls must be mocked. A helper will be added at `src/test/mockTauri.ts` when the
  first command-touching component is tested. Until then, prefer testing data shapes and pure
  helpers.

## Run

```bash
npm run test            # one-shot run, CI-friendly
npm run test:watch      # interactive watch
```

## Out of Scope

- Component visual regression (use desktop launcher + manual pass).
- E2E that drives Tauri commands (deferred — needs Playwright + Tauri test harness ADR).
