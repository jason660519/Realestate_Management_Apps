# ADR-003: Tauri 2.x Adoption

Status: Proposed
Date: 2026-05-17
Owner: Project lead
Related: ADR-002, `.claude/rules/backend/rust-tauri.md`

## Context

Tauri 有兩個主版本同時存在：

- **Tauri 1.x**：穩定、生態成熟、文件多、社群案例豐富
- **Tauri 2.x**：2024 年穩定，重整 capability / permission 系統，新增 mobile target、plugin v2 架構

本專案需要明確選邊，否則：
- `tauri.conf.json` 的 schema 不同（v1 vs v2 不相容）
- Permission / allowlist 系統完全重做（v2 用 capability + scope）
- Plugin API breaking changes
- Crate 版本鎖在 `Cargo.toml` 必須對應

`.claude/rules/backend/rust-tauri.md` 已預設使用 Tauri 2，但缺 ADR 記錄理由。

## Decision

**採用 Tauri 2.x（Cargo `tauri = "2"`，npm `@tauri-apps/api` v2）。**

理由：

1. **Capability model 更貼合本專案需求**
   ADR-002 規定最小權限、http scope 動態注入、shell 全關。Tauri 2 的 capability 系統用 per-window / per-feature 設定，可以做到 v1 allowlist 做不到的細粒度控管。例：可指定「只有 Settings 視窗才有改 server URL 的 capability」。

2. **Plugin v2 對齊本專案 plugin 規劃**
   本專案未來要做跨 app plugin（SayDo、Project-Manager）。Tauri 2 plugin v2 架構支援 Rust + JS 雙端的明確 contract，與 PRD §7 的 plugin 規劃相容性更好。

3. **Mobile target option 保留**
   PRD §8.3 雖把 mobile 列為「Not yet」，但 Tauri 2 提供 iOS / Android target 可選用。選 v1 等於主動關掉這個選項。

4. **v1 即將進入 long-term maintenance**
   Tauri 官方在 2.x 發布後將 1.x 切至維護模式。新專案在 2026 年起跑卻選 v1，等於起跑就是技術債。

## Consequences

### 正面

- 安全模型乾淨：capability 顯式 declare，沒有「default allow」
- Plugin contract 從第一天就是 v2 結構
- 升級壓力小：起手就是當前主流

### 負面 / 成本

- 學習成本：v2 capability JSON schema 比 v1 allowlist 複雜
- 第三方範例少：StackOverflow / 部落格大量內容仍是 v1 範例
- 部分 community plugin 尚未完全移植到 v2，可能需要等或自行 fork
- 文件變動仍頻繁（截至 2026-05），需以官方 docs.rs 為準，社群文章可能過期

### 緩解

- 重要 plugin 採用前先確認 v2 支援狀況（`tauri-plugin-stronghold`、`tauri-plugin-store`、`tauri-plugin-sql` 等）
- Library 文件查詢使用 Context7 MCP，避免被舊版範例誤導

## Capability 設定原則

呼應 ADR-002 的 permission boundary，capability 檔案應遵循：

- 一個 main window 一份 capability（不共用）
- Server URL 寫入 capability 由 dynamic capability runtime API 觸發；驗證可行性後鎖入規格
- 任何 `core:*` capability 都需有 comment 說明使用範圍
- 不使用 `"all": true` 或等價的 wildcard

```json
// 範例：src-tauri/capabilities/main.json
{
  "identifier": "main-capability",
  "description": "Main window: app navigation + property workbench",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "notification:default",
    {
      "identifier": "http:default",
      "allow": [{ "url": "http://192.168.1.*:*/*" }]
    }
  ]
}
```

（http allow list 起始值僅為 placeholder，實際由 config 注入。）

## Dependencies (initial pin)

scaffold 時建議鎖定：

```toml
[dependencies]
tauri = { version = "2", features = [] }       # 視需要加 features
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

具體 features 等 scaffold 落實後再列。

## Alternatives Considered

| 方案 | 拒絕理由 |
|:--|:--|
| Tauri 1.x | 即將進入維護模式；capability model 較粗；起跑即背技術債 |
| Electron | Bundle size、安全模型、記憶體佔用都不符 PRD §6.1 桌面定位 |
| Wails (Go + WebView) | 團隊與專案規則已選 Rust（PRD §6.1） |
| 純 native Swift/Kotlin | 跨平台成本高，且 PRD 沒要求 native UI |

## Open Questions

1. Dynamic capability runtime API 是否支援不重啟換 http scope？需 scaffold 後驗證（rover ADR-002 也記了同題）
2. Tauri 2 IPC 大檔（10MB+ PDF）傳輸效能：直接 invoke 還是 stream？等 Phase 2 document intake 實作時測

## Verification

scaffold 後：

```bash
cargo check --workspace
cargo tauri info  # 應列出 tauri 2.x 版本
```

`Cargo.toml` 中 `tauri` major version 為 `2`；`tauri.conf.json` 的 `$schema` 指向 v2 schema。
