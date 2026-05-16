# Handoff — Phase 1 Hardening

> **產出時間**：2026/05/17
> **產出者**：Codex（與 jason66 對話）
> **接手對象**：下一個 AI coding session
> **承接內容**：Phase 1 desktop foundation 已完成 config persistence 與 frontend shell split，但尚未 commit。
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt

---

```markdown
你是接手 `/Volumes/KLEVV-4T-1/Realestate_Management_Apps` 的 AI coding agent。請用繁體中文回覆；程式碼註解、變數、檔名維持英文。這是 Rust + Tauri 2 + Vite + React + Mantine 的新房產營運桌面 app，舊 `Owner-Property-Management-AI-SPA` 只當 reference，不可直接修改。

## 專案位置

- Repo: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps`
- GitHub remote: `https://github.com/jason660519/Realestate_Management_Apps.git`
- 最新已同步 commit: `df303de docs: 補 Codex 與 Claude workflow 相容指引`
- 目前本地 `main...origin/main` 無 ahead/behind commit，但有未提交 Phase 1 hardening 變更。

## 本輪剛完成但尚未 commit

### Config persistence

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/Cargo.toml`
  - 新增 `toml = "0.8"`。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/services/config.rs`
  - 新增 `ConfigStore`。
  - `load_or_create()` 首次建立 default config。
  - `save()` 寫 temporary file 後 replace `config.toml`。
  - TOML persisted DTO 保持 snake_case；Tauri frontend payload 仍是 camelCase。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/lib.rs`
  - Tauri setup 透過 `app.path().app_data_dir()` 建立 config store。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/state.rs`
  - `AppState` 持有 `ConfigStore`，`update_config()` 會持久化 patch。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/commands/server.rs`
  - 抽出可測的 `check_server_health_for_config()`。
  - 修正空白 server URL 應回 `not_configured`，不是 `offline`。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/commands/plugin.rs`
  - 抽出 `plugin_statuses(&AppConfig)` 供測試。

### Frontend shell split

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/router.tsx`
  - 從 520 行降到 73 行，只保留 TanStack Router route tree。
- 新增 shared components:
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/PageHeader.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/EmptyOperationalState.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/MetricCard.tsx`
- 新增 shell components/context:
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/shell/ShellLayout.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/shell/appData.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/shell/StatusPanel.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/shell/PluginPanel.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/components/shell/navigation.ts`
- 新增 routes:
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/workbench.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/properties.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/documents.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/ai-review.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/tasks.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/integrations.tsx`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/settings.tsx`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/settings.tsx`
  - SayDo / Project-Manager toggles now save through `updateAppConfig()`.
  - Save refreshes shell health/plugin context through `refreshAppData()`.
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/integrations.tsx`
  - Reads plugin state from shell context instead of a separate stale fetch.

### Project process docs

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-config-persistence-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-frontend-shell-split-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/handoffs/handoff-config-persistence-20260517.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/handoffs/handoff-phase-1-hardening-20260517.md`

## 驗證基線

本輪已通過：

```bash
npm run standards:check
npm run typecheck
npm run build
cd src-tauri && cargo fmt --check
cd src-tauri && cargo check
cd src-tauri && cargo test
cd src-tauri && cargo clippy -- -D warnings
git diff --check
```

Rust tests 目前 7 個全過：config create/load、config roundtrip、config patch persistence、empty URL rejection、plugin status mapping、degraded health、not_configured health。

## 關鍵慣例與雷區

- 不要修改舊 `Owner-Property-Management-AI-SPA`。
- Commit message 用 `<type>: <繁體中文描述>`。
- 不要 force push、不要 commit secrets、不要直接改 plugin contract schema 而不問。
- Rust production code 不要 `unwrap()`；測試可用 `expect()`。
- TypeScript strict，禁 `any`。
- Tauri WebView 不直接拿 filesystem、shell、secret、外部 AI provider 權限；敏感副作用走 Rust command。
- Server address 從 config 讀，不要把 IP 寫進 source module business logic。
- 高風險 AI / 房產資料必須 evidence-first；不可假資料、不可 silent fallback。
- 不要在 agent 背景長跑 dev server。

## 建議下一步

1. Review 這批未提交 diff。
2. 若 OK，commit：
   - `feat: 強化桌面設定與前端 shell 結構`
3. 下一個開發任務建議二選一：
   - 補 app icon，取代 placeholder。
   - 補 config-path diagnostics UI，讓 Settings 顯示 config 持久化位置與讀寫狀態。

## 動工前確認指令

```bash
git status --branch --short
git log --oneline -5
npm run standards:check
cd src-tauri && cargo test
```

動工前先跟我確認任務拆解，避免悶頭寫錯方向。
```

---

## 相關文件

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/architecture/ADR-004-local-storage-and-secret.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/product/prd.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-config-persistence-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-frontend-shell-split-2026-05-17.md`
