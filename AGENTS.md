# AGENTS.md

This project follows Company AI App Standards v0.2.

## Required Reading

Before implementation, read:

1. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/ai-engineer-workflow.md`
2. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/ui-design-system.md`
3. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/file-naming-standards.md`
4. `/Volumes/KLEVV-4T-1/Company-AI-App-Standards/docs/multi-app-integration.md`
5. `./DESIGN.md`
6. `./README.md`
7. `./docs/architecture/README.md`
8. `./docs/product/prd.md`

## Project Overrides

Project-specific rules belong in:

- `./DESIGN.md`
- `./docs/architecture/`
- `./README.md`
- `./docs/project-process/ai-workflow-compatibility.md`

If this project must deviate from company standards, create an ADR under `docs/architecture/`.

## Claude/Codex Workflow Compatibility

This repo keeps Claude Code workflow assets under `.claude/`. Codex agents should treat them as repo-local workflow documentation, not as automatically loaded slash commands.

When the user asks for one of the following workflows, first read the matching `.claude/commands/*.md` file and follow its process:

- `/diagnose` or diagnostic loop requests -> `.claude/commands/diagnose.md`
- `/handoff`, handoff prompt, or session transfer requests -> `.claude/commands/handoff.md`
- `/wrap-up`, session wrap-up, or ship-it requests -> `.claude/commands/wrap-up.md`
- commit/push/PR automation requests -> `.claude/commands/commit-push-pr.md`
- daily progress report requests -> `.claude/commands/daily-report.md`

Also read `.claude/rules/*.md` when work touches Rust/Tauri structure, git workflow, handoff/dev-log policy, or long-running dev servers.

See `docs/project-process/ai-workflow-compatibility.md` for the cross-tool mapping.

## Verification

Run before handing work back:

```bash
npm run standards:check
npm run typecheck
npm run build
```

For Rust + Tauri changes, also run the wired Rust checks, usually `cargo fmt --check`, `cargo check`, and `cargo test`.

## Imported Claude Cowork project instructions

1. 建立新的 Rust + Tauri desktop app，不破壞舊 SPA。
2. 把 Docker-based backend/runtime 服務集中到內網 server，桌面 app 只保留必要 local state。
3. 讓 Realestate_Management_Apps、SayDo、Project-Manager 未來能透過明確 plugin contract 互相接入。
4. 延續舊系統已驗證的房產 domain workflow，但重做資訊架構與操作體驗。
5. 高風險 AI 流程必須 evidence-first，不產生假資料，不默默 fallback。
