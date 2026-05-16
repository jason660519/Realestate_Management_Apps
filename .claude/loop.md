# Session maintenance loop (bare `/loop`)

You are running the **in-session maintenance** loop for this repository. Stay within scope; do not start unrelated features.

## Priority order

1. **Continue unfinished work** from this conversation (commits, tests, fixes) if any were explicitly started.
2. **Current branch PR** (if one exists for the checked-out branch):
   - Fetch PR metadata and checks (`gh pr view`, `gh pr checks`).
   - If CI is failing: diagnose from logs; apply **minimal** fixes (types, lint, clippy). Avoid product behavior changes without explicit user intent.
   - If there are **unresolved review threads**: address each with small commits.
   - If merge conflicts with base: attempt a clean rebase/merge resolution.
3. **When idle** (nothing above pending): run light cleanup only if clearly safe—note stale todos in chat, suggest next steps. Do **not** push, merge, delete branches, or close issues unless this session already authorized that action.

## Guardrails

- **Rust**: `cargo fmt --check` + `cargo clippy -- -D warnings` must pass.
- **TypeScript strict**; no `any`.
- **Irreversible GitHub actions** (merge, delete remote branch, close issues/PRs): only if the user or prior transcript explicitly approved; otherwise **report and stop**.
- Prefer **`gh` CLI** and repo scripts over ad-hoc API calls.
- Server address from config, never hardcode.

## Output

Each iteration: one short status line—what you checked, what you changed, what is blocked—unless there is actionable work that needs more detail.
