# Handoff — AI Workflow Compatibility

> **產出時間**：2026/05/17  
> **產出者**：Codex（GPT-5，與 Jason 對話）  
> **接手對象**：下一個 Claude 或 Codex session  
> **承接內容**：承接 Realestate_Management_Apps Phase 1 scaffold 後的 Claude/Codex workflow compatibility 文件變更與下一步工作。  
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt。

---

````markdown
你是接手 `/Volumes/KLEVV-4T-1/Realestate_Management_Apps` 的 AI coding agent。請用繁體中文回覆；程式碼註解、變數、檔名維持英文。這是 Rust + Tauri 2 + Vite + React + Mantine 的新房產營運桌面 app，舊 `Owner-Property-Management-AI-SPA` 只當 reference，不可直接修改。

## 專案位置

- Repo: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps`
- GitHub remote: `https://github.com/jason660519/Realestate_Management_Apps.git`
- 最新已提交 commit: `a4a5491 feat: scaffold Tauri 2.0 app with Vite, React, and Mantine`
- 目前未提交變更：
  - `M AGENTS.md`
  - `?? docs/project-process/ai-workflow-compatibility.md`

## 目前已完成

1. Phase 0 architecture baseline 已提交在 `dc354cc`。
2. Phase 1 scaffold 已提交在 `a4a5491`：
   - Tauri config: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/tauri.conf.json`
   - Rust command skeleton: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/commands/`
   - Frontend shell: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/router.tsx`
   - Tauri API wrapper: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/api/tauri.ts`
   - README verification instructions: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/README.md`
3. 本 session 新增/修改但尚未 commit：
   - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/AGENTS.md`
   - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/ai-workflow-compatibility.md`
4. 這兩個 workflow compatibility 變更的目的：
   - 告訴 Codex：`.claude/commands/*.md` 不是自動 slash commands，但必須當 repo-local workflow docs 讀取。
   - 映射 `/diagnose`、`/handoff`、`/wrap-up`、commit/push/PR、daily report 到 `.claude/commands/*.md`。
   - 提醒 Codex 也要沿用 `.claude/rules/general.md`、`.claude/rules/backend/rust-tauri.md`、`.claude/rules/claude-code-background-shell.md`。

## 已驗證事實

- `AGENTS.md` 目前已包含 `Claude/Codex Workflow Compatibility` 區塊，並指向 `docs/project-process/ai-workflow-compatibility.md`。
- `docs/project-process/ai-workflow-compatibility.md` 已列出 Claude/Codex 對應表與 command equivalents。
- `package.json` 目前 scripts 包含：
  - `npm run standards:check`
  - `npm run typecheck`
  - `npm run build`
  - `npm run dev`
  - `npm run tauri:dev`
- `README.md` 已記錄 Phase 1 scaffold 與完整 verification 指令。
- `src/api/tauri.ts` 有非 Tauri preview fallback，所以單純 Vite 預覽會顯示 preview/offline，不會拋 raw Tauri runtime error。
- 產品方向在 `docs/product/prd.md`：Rust + Tauri desktop、不破壞舊 SPA、內網 server、evidence-first、SayDo / Project-Manager 明確 plugin contract。

## 驗證基線

接手後先跑：

    git status --short
    git log --oneline -5
    npm run standards:check
    npm run typecheck
    npm run build
    cd src-tauri
    cargo fmt --check
    cargo check
    cargo test
    cargo clippy -- -D warnings

本 session 已跑過並通過：

- `npm run standards:check`
- `npm run typecheck`
- `npm run build`
- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`

## 關鍵慣例與雷區

- 必讀：
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/AGENTS.md`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/CLAUDE.md`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/.claude/rules/general.md`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/.claude/rules/backend/rust-tauri.md`
  - `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/.claude/rules/claude-code-background-shell.md`
- Git commit message 用 `<type>: <繁體中文描述>`。
- 不要 force push、不要 commit secrets、不要直接改 plugin contract schema 而不問。
- Rust 必須 `cargo fmt`、`cargo clippy -- -D warnings` 全過；production code 不要 `unwrap()`。
- TypeScript strict，禁 `any`。
- Tauri WebView 不直接拿 filesystem、shell、secret、外部 AI provider 權限；敏感副作用走 Rust command。
- Server address 從 config 讀，不要把 IP 寫進 source module 的 business logic。
- 高風險 AI / 房產資料必須 evidence-first；不可假資料、不可 silent fallback。
- `.claude/rules/claude-code-background-shell.md` 說不要用 Claude/Codex 背景長跑 dev server 造成 log 爆量。若需要 dev server，讓使用者自己開 terminal，或短暫啟動後確實 kill。

## 下一步建議

1. 先 review 目前兩個未提交 workflow compatibility 變更。
2. 若確認 OK，commit/push：
   - 建議 commit message：`docs: 補 Codex 與 Claude workflow 相容指引`
3. 接著進 Phase 1 後續硬化：
   - 把 `src/router.tsx` 拆成 `src/routes/`、`src/components/`，避免單檔過大。
   - 將 app config 從目前 in-memory `AppState` 落到 app data config file（對齊 ADR-004）。
   - 補 Rust tests for config patch / plugin list / server health degraded response。
   - 補正式 app icon，取代 `src-tauri/icons/icon.png` 目前的 placeholder。
   - 視需要補 `docs/project-process/dev-logs/dev-phase-1-scaffold-2026-05-17.md`。

## 延後 / 待辦

- 真正 server deployment 尚未做；`rick@192.168.1.6` 的 OS/Docker/GPU/backup 仍待確認。
- Postgres/PostgREST service specs 已有草稿，但 compose files 還沒建。
- SayDo / Project-Manager integration 目前只有 contract docs 與 UI placeholder，尚未實作真 transport。
- Property CRUD、Document intake、Evidence review workflow 都是 Phase 2+。

## 驗收門檻

本次 workflow compatibility 收尾的完成條件：

- `AGENTS.md` 保留 Claude/Codex compatibility 指引。
- `docs/project-process/ai-workflow-compatibility.md` 存在，內容說明 `.claude/commands` 和 `.claude/rules` 在 Codex 端如何使用。
- `npm run standards:check` 通過。
- 若 commit，使用：`docs: 補 Codex 與 Claude workflow 相容指引`。

動工前先跟我確認任務拆解，避免悶頭寫錯方向。
````

---

## 相關文件

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/AGENTS.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/ai-workflow-compatibility.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/.claude/commands/handoff.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/.claude/rules/general.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/product/prd.md`
