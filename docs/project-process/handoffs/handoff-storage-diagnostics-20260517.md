# Handoff — Storage Diagnostics

> **產出時間**：2026/05/17
> **產出者**：Codex（與 jason66 對話）
> **接手對象**：下一個 AI coding session
> **承接內容**：同事已 commit/push Phase 1 hardening，本輪續作 storage diagnostics，尚未 commit。
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt

---

```markdown
你是接手 `/Volumes/KLEVV-4T-1/Realestate_Management_Apps` 的 AI coding agent。請用繁體中文回覆；程式碼註解、變數、檔名維持英文。這是 Rust + Tauri 2 + Vite + React + Mantine 的新房產營運桌面 app，舊 `Owner-Property-Management-AI-SPA` 只當 reference，不可直接修改。

## 專案位置

- Repo: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps`
- GitHub remote: `https://github.com/jason660519/Realestate_Management_Apps.git`
- 已確認同事 commit/push 成功：`673b9d6 feat: 實作配置持久化並重構前端架構`
- `git fetch origin` 後 `git status --branch --short` 顯示 `main...origin/main`，`origin/main...HEAD` 無輸出。

## 本輪剛完成但尚未 commit

新增 Settings 的 local storage diagnostics：

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/models.rs`
  - 新增 `StorageDiagnostics`。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/services/config.rs`
  - 新增 `ConfigStore::diagnostics()`。
  - 回傳 app data dir、config path、config 是否存在、是否可讀、檔案大小、錯誤訊息。
  - diagnostics 是唯讀，不寫 probe file。
  - 新增 `diagnostics_reports_created_config_file` Rust test。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/state.rs`
  - 新增 `storage_diagnostics()`。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/commands/app.rs`
  - 新增 Tauri command `get_storage_diagnostics`。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/commands/mod.rs`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/src/lib.rs`
  - 註冊新 command。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/api/tauri.ts`
  - 新增 `StorageDiagnostics` type 與 `getStorageDiagnostics()`。
  - Vite preview fallback 會說明 config.toml 只在 Tauri runtime 建立。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/routes/settings.tsx`
  - 新增 `Local storage` panel。
  - 顯示 app data directory、config file、存在/可讀/bytes badge、error。
  - 載入與 save 後都會刷新 diagnostics。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src/styles.css`
  - 新增 `.path-text { overflow-wrap: anywhere; }` 避免長路徑溢出。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-storage-diagnostics-2026-05-17.md`
  - 本輪 dev-log。
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/handoffs/handoff-storage-diagnostics-20260517.md`
  - 本 handoff。

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

Rust tests 現在 8 個全過。

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

1. Review 這批 storage diagnostics diff。
2. 若 OK，commit：
   - `feat: 顯示本機設定儲存診斷`
3. 下一個開發任務建議：
   - 補正式 Tauri app icon，取代 placeholder。
   - 或補 frontend Settings smoke test harness。

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
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-storage-diagnostics-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/handoffs/handoff-phase-1-hardening-20260517.md`
