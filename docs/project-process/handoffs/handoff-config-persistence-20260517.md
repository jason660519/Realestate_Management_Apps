# Handoff — Config Persistence

> **產出時間**：2026/05/17
> **產出者**：Codex（與 jason66 對話）
> **接手對象**：下一個 AI coding session
> **承接內容**：Rust/Tauri desktop config persistence 已完成但尚未 commit，下一步建議拆分 frontend shell 與補 plugin enablement UI。
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt

---

```markdown
你是接手 `/Volumes/KLEVV-4T-1/Realestate_Management_Apps` 的 AI coding agent。請用繁體中文回覆；程式碼註解、變數、檔名維持英文。這是 Rust + Tauri 2 + Vite + React + Mantine 的新房產營運桌面 app，舊 `Owner-Property-Management-AI-SPA` 只當 reference，不可直接修改。

## 專案位置

- Repo: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps`
- GitHub remote: `https://github.com/jason660519/Realestate_Management_Apps.git`
- 最新已同步 commit: `df303de docs: 補 Codex 與 Claude workflow 相容指引`
- `git fetch origin` 後 `git log --oneline --left-right --cherry-pick origin/main...HEAD` 無輸出，表示本地 `main` 與 `origin/main` 無 ahead/behind commit。

## 目前未提交變更

本輪完成 Phase 1 hardening：把 app config 從 in-memory `AppState` 落到 Tauri app data `config.toml`。

- `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock`
  - 新增 `toml = "0.8"`。
- `src-tauri/src/services/mod.rs`
  - 新增 service module root。
- `src-tauri/src/services/config.rs`
  - 新增 `ConfigStore`。
  - 使用 `ConfigStore::from_app_data_dir(app_data_dir)` 指向 `config.toml`。
  - `load_or_create()` 首次建立 default config。
  - `save()` 用 temporary file replace 寫回。
  - TOML persisted DTO 使用 snake_case，避免污染 Tauri API 的 camelCase JSON shape。
- `src-tauri/src/lib.rs`
  - 啟動時在 `.setup()` 內用 `app.path().app_data_dir()` 建立 `ConfigStore` 並 `app.manage(AppState::load(store)?)`。
- `src-tauri/src/state.rs`
  - `AppState` 改成持有 `ConfigStore`。
  - `update_config()` 先 validate/clone patch，再 save，最後更新 mutex 內 config。
  - 空白 server base URL 會回 `AppError::InvalidInput`。
- `src-tauri/src/errors.rs`
  - 新增 `ConfigStorage` error kind。
- `src-tauri/src/models.rs`
  - `AppConfig` / `ServerConfig` / `PluginConfig` 新增 `PartialEq, Eq`，方便測試。
- `src-tauri/src/commands/plugin.rs`
  - 抽出 `plugin_statuses(&AppConfig)`，讓 plugin flag mapping 可測。
- `src-tauri/src/commands/server.rs`
  - 抽出 `check_server_health_for_config(AppConfig)`，讓 health behavior 可測。
  - server URL 現在會先 `trim()` 再判斷是否 empty，再移除尾端 `/`。
- `src/router.tsx`
  - Settings saved notification 改成 `Server configuration saved to local app data.`。
- `docs/project-process/dev-logs/dev-config-persistence-2026-05-17.md`
  - 記錄本輪完成、技術筆記與下一步。
- `docs/project-process/handoffs/handoff-config-persistence-20260517.md`
  - 本 handoff。

## 已驗證事實

- `AGENTS.md` 要求 implementation 前讀公司標準、`DESIGN.md`、`README.md`、`docs/architecture/README.md`、`docs/product/prd.md`。
- `CLAUDE.md` 規定 Rust 必須 `cargo fmt` + `cargo clippy` 全過、TS strict 禁 `any`、server address 不可 hardcode 到 business logic、AI 高風險流程 evidence-first。
- `.claude/rules/backend/rust-tauri.md` 規定 app 設定讀寫走 Rust invoke，Config 路徑是 app data 的 `config.toml`，server address 從 config 讀。
- `docs/architecture/ADR-004-local-storage-and-secret.md` 決策：Config 存 TOML，透過 `services/config.rs` 讀寫，使用 temp file 再 rename。
- Tauri 2 docs 已查過：Rust backend 可透過 `app.path().app_data_dir()` 取得 app data dir；需要 `tauri::Manager`。

## 驗證基線

本輪已通過：

```bash
npm run standards:check
npm run typecheck
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
git diff --check
```

`cargo test` 現有 7 個 Rust tests 全過：

- config create/load snake_case TOML
- config save/reload roundtrip
- config patch persistence
- empty server base URL rejection
- plugin status mapping
- degraded health response for HTTP 503
- not_configured health response for empty URL

## 關鍵慣例與雷區

- 不要直接修改舊 `Owner-Property-Management-AI-SPA`。
- Git commit message 用 `<type>: <繁體中文描述>`。
- 不要 force push、不要 commit secrets、不要直接改 plugin contract schema 而不問。
- Rust production code 不要 `unwrap()`；測試可用 `expect()`。
- Tauri WebView 不直接拿 filesystem、shell、secret、外部 AI provider 權限；敏感副作用走 Rust command。
- Server address 從 config 讀，不要把 IP 寫進 source module business logic。`AppConfig::default()` 目前仍依 scaffold baseline 保留 default URL。
- 高風險 AI / 房產資料必須 evidence-first；不可假資料、不可 silent fallback。
- `.claude/rules/claude-code-background-shell.md` 禁止在 agent 背景長跑 dev server。若需要 dev server，讓使用者自己開 terminal，或短暫啟動後確實 kill。
- `.claude/rules/frontend/webview.md` 目前在 `CLAUDE.md` 被列為索引，但檔案不存在；本輪未補，避免擴大範圍。

## 下一步任務拆解

建議下一步先做 frontend 結構拆分，不要急著進 Property CRUD：

1. 拆 `src/router.tsx`
   - `src/routes/root.tsx`
   - `src/routes/workbench.tsx`
   - `src/routes/properties.tsx`
   - `src/routes/documents.tsx`
   - `src/routes/ai-review.tsx`
   - `src/routes/tasks.tsx`
   - `src/routes/integrations.tsx`
   - `src/routes/settings.tsx`
   - `src/components/shell/`
   - `src/components/status/`
2. 補 Settings plugin toggles
   - 讓 SayDo / Project-Manager enabled flags 可以從 UI 修改。
   - 儲存時呼叫既有 `updateAppConfig({ saydoEnabled, projectManagerEnabled })`。
   - 儲存後 refresh `listPlugins()`，讓 aside/plugin registry 狀態一致。
3. 再補 dev-log
   - `docs/project-process/dev-logs/dev-frontend-shell-split-2026-05-17.md`

## 延後 / 待辦

- 真正 server deployment 尚未做；`rick@192.168.1.6` 的 OS/Docker/GPU/backup 仍待確認。
- SQLite state、keychain secret、rolling logs 還未實作；ADR-004 只完成 Config/TOML 的第一段。
- Postgres/PostgREST compose files 還沒建。
- SayDo / Project-Manager integration 目前只有 contract docs 與 UI placeholder，尚未實作真 transport。
- Property CRUD、Document intake、Evidence review workflow 都是 Phase 2+。

## 驗收門檻

若要收本輪 config persistence：

- `config.toml` 會在 Tauri app data dir 首次建立。
- Settings 更新 server base URL 後可持久化到 TOML。
- TOML key 保持 snake_case；Tauri frontend payload 保持 camelCase。
- Rust tests、frontend typecheck/build、standards check、clippy 全通過。
- 建議 commit message：`feat: 持久化桌面 app 設定檔`

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
