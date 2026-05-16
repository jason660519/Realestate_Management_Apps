# CLAUDE.md

本檔只放「每個 session 都該知道、且沒別處可放」的規則。
架構、Rust/Tauri 慣例、服務邊界 → 見 `.claude/rules/`。
產品需求 → `docs/product/prd.md`。
架構決策 → `docs/architecture/ADR-*.md`。

## 邊界

**絕對不做**（hooks / pre-commit 攔截）：background 起 dev server、引入 `unsafe` 未經 review、commit secrets
**必須先問**：force push、刪 migration 檔、直接 merge 到 main、變更 plugin contract schema
**自主決定**：重構路徑、測試策略、命名細節、Rust module 拆分

## 硬性規定

- Rust：`cargo fmt` + `cargo clippy` 全過，禁止 `#[allow(clippy::*)]` 除非 ADR 記錄理由
- TypeScript strict（前端），禁 `any`
- 文檔/臨時檔不能放根目錄，單檔不超過 500 行
- Server address 從 config 讀，禁止 hardcode IP
- AI 高風險流程必須 evidence-first，不可產生假資料
- **新規則優先加到 `.claude/rules/`**，本檔只放 pointer

## Rules 索引（有疑問先讀）

| 主題 | 檔案 |
|---|---|
| 命名、檔案組織、Git、進度更新 | `.claude/rules/general.md` |
| Rust + Tauri 慣例、command surface、security | `.claude/rules/backend/rust-tauri.md` |
| 前端框架慣例（scaffold 後補） | `.claude/rules/frontend/webview.md` |
| Claude Code background shell 漏水點 | `.claude/rules/claude-code-background-shell.md` |

## 啟動（scaffold 後更新）

```bash
# Development
cargo tauri dev          # Desktop app + WebView hot reload
cargo test               # Rust unit tests
npm run dev              # Frontend only (in separate terminal)

# Checks
cargo fmt --check
cargo clippy -- -D warnings
npm run typecheck
npm run standards:check
```

## 工具優先序

查 Rust 函式關係 → `cargo doc --open` 或 grep；library 文件 → Context7 MCP 或 docs.rs；瀏覽器測試 → Playwright CLI；完整 token 指南 → 待建立

## 語言偏好

| 場景 | 語言 |
|---|---|
| 程式碼註解 / 變數命名 | 英文 |
| Commit 訊息 / 文檔 / UI 文字 | 繁體中文 |

## 其他

- 跨 app plugin contract 變更需三方（Realestate、SayDo、Project-Manager）同步更新文件
- Server 服務異動需同步更新 `docs/deployment/internal-server-plan.md`
