# 通用開發規則

> Rust/Tauri 慣例見 `.claude/rules/backend/rust-tauri.md`。前端慣例見 `.claude/rules/frontend/webview.md`。啟動指令見 `CLAUDE.md`。

---

## 命名規範

| 文件類型 | 規則 | 範例 |
|:--|:--|:--|
| Rust module | snake_case | `property_service.rs` |
| Rust struct/enum | PascalCase | `PropertyDetail`, `DocumentStage` |
| Rust trait | PascalCase | `EvidenceProvider` |
| TypeScript React 組件 | PascalCase.tsx | `PropertyCard.tsx` |
| TypeScript Hooks | camelCase + `use` 前綴 | `useServerHealth.ts` |
| TypeScript 工具函數 | camelCase.ts | `formatAddress.ts` |
| 資料夾 | kebab-case | `document-intake/` |
| 文檔檔名 | 英文 kebab-case | `internal-server-plan.md` |
| 單元測試（Rust） | 同 module 內 `#[cfg(test)]` | `mod tests { ... }` |
| 整合測試（Rust） | `tests/` 目錄 | `tests/server_health.rs` |
| 前端測試 | `.test.ts(x)` / `.spec.ts` | `PropertyCard.test.tsx` |
| Migration | `YYYYMMDDHHMMSS_description.sql` | `20260517120000_create_properties.sql` |
| ADR | `ADR-###-kebab-case.md` | `ADR-002-tauri-service-architecture.md` |

---

## Git 工作流

**Commit**: `<type>: <繁體中文描述>` — feat / fix / docs / refactor / style / test / chore

**分支**: `feature/xxx` / `fix/xxx` / `docs/xxx` / `chore/xxx`（勿在 `main` 直接開發）

**合併策略**: Squash and merge 為預設

---

## 程式碼風格

### Rust
- `cargo fmt` 預設設定（4 空格縮排）
- `cargo clippy -- -D warnings` 必須全過
- 禁止 `unwrap()` 在 production code（測試可用）
- Error handling: 使用 `thiserror` 定義 domain error，`anyhow` 限 binary crate

### TypeScript（前端）
- 2 空格縮排，單引號，行尾不留空白，檔案結尾一空行
- TypeScript strict mode，禁止 `any`
- Interface PascalCase，`export type`

---

## 語言偏好

| 場景 | 語言 |
|:--|:--|
| 程式碼註解 / 變數命名 | 英文 |
| Commit 訊息 / 文檔 / UI 文字 | 繁體中文 |

---

## 檔案組織

**新增檔案位置**：

- Tauri Rust commands → `src-tauri/src/commands/`
- Rust services/domain → `src-tauri/src/services/`
- Rust models/types → `src-tauri/src/models/`
- 前端頁面 → `src/pages/` 或 `src/routes/`
- 前端組件 → `src/components/`
- 前端工具 → `src/lib/`
- 文檔 → `docs/` 下對應分類
- Dev logs → `docs/project-process/dev-logs/`
- Handoffs → `docs/project-process/handoffs/`

**禁止**：根目錄放文檔/臨時檔、單檔超過 500 行、非英文檔名/路徑名

---

## 進度追蹤

每次完成工作後，產出 dev-log 存到 `docs/project-process/dev-logs/dev-{topic}-{YYYY-MM-DD}.md`。

格式包含：
1. 本次完成任務清單（含交付物）
2. 技術困難與解法
3. 下次優先工作

---

## Session 結束規範

Session 收尾時，若有以下任一交付，必須呼叫 `/handoff` command：

- 有 PR 被 merged
- 新增結構性檔案（ADR、migration、新 module）
- 有明確「下一步任務」尚未動工

---

## 除錯備註

- Tauri dev 模式 WebView 開啟 DevTools：`Cmd+Option+I`（macOS）
- Rust panic 堆疊：`RUST_BACKTRACE=1 cargo tauri dev`
- Server 連線問題：先 `curl http://{server}/health` 確認 server 狀態
