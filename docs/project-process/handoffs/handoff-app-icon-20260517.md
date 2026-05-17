# Handoff — App Icon

> **產出時間**：2026/05/17
> **產出者**：Codex（與 jason66 對話）
> **接手對象**：下一個 AI coding session
> **承接內容**：同事已 commit/push Phase 1 hardening，本輪續作 storage diagnostics 與 app icon，尚未 commit。
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt

---

```markdown
你是接手 `/Volumes/KLEVV-4T-1/Realestate_Management_Apps` 的 AI coding agent。請用繁體中文回覆；程式碼註解、變數、檔名維持英文。這是 Rust + Tauri 2 + Vite + React + Mantine 的新房產營運桌面 app，舊 `Owner-Property-Management-AI-SPA` 只當 reference，不可直接修改。

## 專案位置

- Repo: `/Volumes/KLEVV-4T-1/Realestate_Management_Apps`
- GitHub remote: `https://github.com/jason660519/Realestate_Management_Apps.git`
- 已確認同事 commit/push 成功：`673b9d6 feat: 實作配置持久化並重構前端架構`
- 目前本地有未提交變更：storage diagnostics + app icon。

## 本輪剛完成但尚未 commit

### Storage diagnostics

- `src-tauri/src/models.rs`
  - 新增 `StorageDiagnostics`。
- `src-tauri/src/services/config.rs`
  - 新增 `ConfigStore::diagnostics()`。
  - 回傳 app data dir、config path、config 是否存在、是否可讀、檔案大小、錯誤訊息。
  - diagnostics 是唯讀，不寫 probe file。
- `src-tauri/src/commands/app.rs`
  - 新增 Tauri command `get_storage_diagnostics`。
- `src/api/tauri.ts`
  - 新增 `StorageDiagnostics` type 與 `getStorageDiagnostics()`。
- `src/routes/settings.tsx`
  - 新增 `Local storage` panel，顯示設定檔實際位置與讀寫狀態。
- `src/styles.css`
  - 新增 `.path-text` 避免長路徑溢出。
- `docs/project-process/dev-logs/dev-storage-diagnostics-2026-05-17.md`
- `docs/project-process/handoffs/handoff-storage-diagnostics-20260517.md`

### App icon

- `src-tauri/icons/app-icon.svg`
  - 新增可重現 source artwork。
- 已執行：
  - `npm run tauri icon src-tauri/icons/app-icon.svg`
- 生成 icon assets：
  - `src-tauri/icons/icon.png`
  - `src-tauri/icons/icon.icns`
  - `src-tauri/icons/icon.ico`
  - `src-tauri/icons/32x32.png`
  - `src-tauri/icons/64x64.png`
  - `src-tauri/icons/128x128.png`
  - `src-tauri/icons/128x128@2x.png`
  - Windows Store square/logo PNGs
  - iOS icon PNGs
  - Android mipmap icon PNG/XMLs
- `src-tauri/tauri.conf.json`
  - `bundle.icon` 從空陣列改為明確引用 desktop bundle icons。
- `docs/project-process/dev-logs/dev-app-icon-2026-05-17.md`
- `docs/project-process/handoffs/handoff-app-icon-20260517.md`

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

Rust tests 現在 8 個全過。主要 icon `src-tauri/icons/icon.png` 已檢視，為 512x512 PNG。

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

1. Review storage diagnostics + app icon diff。
2. 若 OK，commit：
   - `feat: 顯示設定儲存診斷並更新 app icon`
3. 下一個開發任務建議：
   - 補 Settings smoke test harness。
   - 或開始 Phase 2 前的 property/document boundary doc/checklist。

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

- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-storage-diagnostics-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/docs/project-process/dev-logs/dev-app-icon-2026-05-17.md`
- `/Volumes/KLEVV-4T-1/Realestate_Management_Apps/src-tauri/icons/app-icon.svg`
