# AI Workflow Compatibility

Status: Repo-local workflow guide  
Applies to: Claude Code, Codex, and other AI coding agents  
Related: `AGENTS.md`, `.claude/commands/`, `.claude/rules/`

## Purpose

This repo contains Claude Code workflow files under `.claude/`. Codex can read and follow those files, but Codex does not automatically expose them as Claude-style slash commands. To keep both tools aligned, treat `.claude/` as shared repo-local workflow documentation.

## Mapping

| Repo asset | Claude Code behavior | Codex behavior |
|---|---|---|
| `AGENTS.md` | Project instructions when supported | Primary repo instruction file |
| `CLAUDE.md` | Claude-specific project guidance | Reference document when relevant |
| `.claude/rules/*.md` | Claude rule files | Read manually before matching work |
| `.claude/commands/*.md` | Slash-command workflow source | Read manually when user asks for that workflow |
| `.claude/memory/*.md` | Claude project memory | Reference only; do not assume it is current without verification |
| `.claude/hooks/*.sh` | Claude hook scripts | Reference only unless explicitly invoked by a repo script |

## Command Equivalents

When the user uses these words in Codex, read and follow the matching command file before acting:

| User wording | Read |
|---|---|
| `/diagnose`, `diagnose`, difficult bug loop | `.claude/commands/diagnose.md` |
| `/handoff`, handoff prompt, next session prompt, 接手 | `.claude/commands/handoff.md` |
| `/wrap-up`, 收尾, ship it, session 結束 | `.claude/commands/wrap-up.md` |
| commit push PR, full-auto git flow | `.claude/commands/commit-push-pr.md` |
| daily report, 今日進度報告 | `.claude/commands/daily-report.md` |

If a command file says to create a dev log, handoff, or project-process artifact, create it under `docs/project-process/` using the file naming rules in `.claude/rules/general.md`.

## Rules Codex Should Reuse

Codex should read `.claude/rules/general.md` for:

- Git commit message format.
- TypeScript and Rust style conventions.
- Dev log and handoff expectations.
- File placement and naming.

Codex should read `.claude/rules/backend/rust-tauri.md` for:

- Tauri command surface structure.
- Rust service/model/module placement.
- Error handling and Tauri security boundaries.

Codex should read `.claude/rules/claude-code-background-shell.md` before starting long-running dev servers or watchers. Even though this file is Claude-specific, the underlying risk still applies: do not leave unbounded background process logs running without a cleanup plan.

## When To Promote A Workflow

Keep `.claude/commands/*.md` as the source when a workflow is mostly project-specific.

Promote a workflow into a Codex skill only when it is reusable across multiple repos. Candidate skills:

- `handoff-writer`
- `diagnostic-loop`
- `session-wrap-up`

Until then, `AGENTS.md` plus this compatibility guide is the bridge.

## Verification

When changing workflow files:

```bash
npm run standards:check
```

If the change also touches source code, run the full verification set from `AGENTS.md`.
