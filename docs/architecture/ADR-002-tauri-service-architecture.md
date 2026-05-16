# ADR-002: Tauri Service Architecture

Status: Proposed
Date: 2026-05-17
Owner: Project lead
Supersedes: —
Related: ADR-001, `docs/product/prd.md` §6 §14, `.claude/rules/backend/rust-tauri.md`

## Context

桌面 app 是 Rust + Tauri，重型服務在內網 server `rick@192.168.1.6`。在 scaffold 之前需要明確切分三層權限與責任：

- WebView（前端 UI 層）
- Rust command surface（Tauri `invoke`）
- Internal server（HTTP）

否則 `tauri.conf.json` allowlist、command 模組結構、前端 fetch 範圍、secret 流向都無法定案。本 ADR 鎖定 v0 command surface 與 permission boundary，作為 Phase 1 scaffold 的硬性輸入。

## Decision

採用三層分離架構，每層責任明確、不互相代理。

### Layer 1: WebView (Frontend)

- 純 UI render、表單驗證、本地 component state
- 不直接 access filesystem、shell、secret、external API
- 所有副作用走 `invoke` 進入 Rust 層
- HTTP 通訊一律經 Rust（不允許 WebView 直接 `fetch` 到 server 或外網）
- 例外：dev 模式下 health check 的明文 GET 可暫由 WebView 直發；正式版收回

### Layer 2: Rust Command Surface (Tauri invoke)

責任：app-local state、權限敏感操作、所有對外 HTTP 入口。

Module 配置（`src-tauri/src/commands/`）：

```
app.rs           config 讀寫
server.rs        health check, base URL 管理
files.rs         filesystem dialog 與 import
property.rs      local cache 讀寫 + server proxy
document.rs      document import + processing trigger
plugin.rs        plugin registry CRUD
secret.rs        secret 寫入（不回傳值到 WebView）
diagnostics.rs   log export、版本資訊
```

### Layer 3: Internal Server (HTTP)

責任：canonical data、重型運算、跨 app 共用服務。

- Property / Document / Task canonical CRUD
- Document processing pipeline（OCR、AI extraction）
- GIS lookup / asset generation
- Search / indexing
- AI model routing 與 stage trace 寫入
- Long-running job 進度推送（協定待定，候選：SSE / WebSocket / polling）

## Command Surface (v0 skeleton)

實際簽名等 scaffold 時定型。以下為 v0 列表，作為 module 切分依據：

```rust
// commands/app.rs
get_app_config() -> AppConfig;
update_app_config(patch: AppConfigPatch) -> AppConfig;

// commands/server.rs
check_server_health() -> ServerHealth;
get_server_base_url() -> String;
set_server_base_url(url: String) -> Result<(), AppError>;

// commands/files.rs
pick_files(filters: FileFilters) -> Vec<PathBuf>;
import_document(path: PathBuf, property_id: Option<Uuid>) -> ImportResult;

// commands/property.rs
list_properties_cached() -> Vec<PropertySummary>;
fetch_property_detail(id: Uuid) -> PropertyDetail;
save_property_draft(draft: PropertyDraft) -> Uuid;
sync_draft_to_server(draft_id: Uuid) -> Result<Uuid, AppError>;

// commands/document.rs
list_documents(property_id: Option<Uuid>) -> Vec<DocumentSummary>;
trigger_processing(document_id: Uuid) -> Uuid; // returns run_id
get_processing_run(run_id: Uuid) -> ProcessingRun;

// commands/plugin.rs
list_plugins() -> Vec<PluginStatus>;
set_plugin_enabled(id: PluginId, enabled: bool) -> Result<(), AppError>;

// commands/secret.rs
set_secret(scope: SecretScope, key: String, value: String) -> Result<(), AppError>;
list_secret_keys(scope: SecretScope) -> Vec<String>; // values 不外傳
// 注意：刻意不提供 get_secret 給 WebView。Rust 內部使用時走 service 層 helper
```

前端 binding 由 `tauri-specta` 或手寫 TypeScript types 對齊。具體工具選擇待前端框架 ADR 後決定。

## Permission Boundary

Tauri 2.x capability 設定原則（最小權限）：

| 能力 | 範圍 |
|:--|:--|
| `fs` | `$APPDATA/<app>/*` + 使用者透過 dialog 選取的路徑 |
| `http` | `${config.server.base_url}/*`（啟動時動態注入） |
| `shell` | 全關 |
| `dialog` | open + save |
| `notification` | 啟用 |
| `clipboard-read` / `clipboard-write` | 全關（避免 secret 外洩） |

硬性規則：
- `tauri.conf.json` 不允許 `"all": true`
- 所有 capability 用 explicit allow list
- `shell.execute` / `shell.open` 都不開

## Error Model

所有 command 回傳 `Result<T, AppError>`，使用 `thiserror`：

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Server unreachable: {0}")]
    ServerUnreachable(String),
    #[error("Server returned {status}: {message}")]
    ServerError { status: u16, message: String },
    #[error("Local IO error: {0}")]
    LocalIo(#[from] std::io::Error),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Plugin error: {plugin}: {reason}")]
    PluginError { plugin: String, reason: String },
}
```

Serde 序列化到 WebView 時 `rename_all = "camelCase"`，並加 `kind` discriminator field。

## Consequences

- 所有敏感操作走 invoke，前端開發節奏比純 web app 慢，但 secret 與權限收斂明確
- Server base URL 變更需熱更新 http scope；需驗證 Tauri 2 capability runtime API 是否支援不重啟
- 維護 invoke binding 是固定成本，建議用 `tauri-specta` 自動產生 TS types
- WebView 拿不到 secret 原值。AI provider key 由 Rust service 注入 server 請求 header

## Alternatives Considered

| 方案 | 拒絕理由 |
|:--|:--|
| WebView 直接打 server | Secret 與 fs 還是要走 Rust，雙路徑混合更難維護 |
| WebView 直接 access fs（broad scope） | 違反 PRD §14 與安全原則 |
| 全部 Rust（含 property CRUD 邏輯） | Server 是 canonical source，重複實作會分裂事實來源 |

## Open Questions

1. Long-running job 進度推送協定（SSE / WebSocket / polling）由 server 規格決定。
2. Server URL 變更時 http scope 熱更新是否需重啟，待 Tauri 2 capability runtime API 驗證。
3. Secret 實際存放（OS keychain via `keyring` crate vs `tauri-plugin-stronghold`），由 ADR-004 storage 決策定。
4. 前端 binding 工具（`tauri-specta` vs 手寫）等前端框架 ADR 後選定。

## Verification

Scaffold 後驗證：

```bash
cargo check --workspace
cargo clippy -- -D warnings
cargo test --workspace
```

Review checklist：
- [ ] `tauri.conf.json` 無 `"all": true`
- [ ] 無 `#[allow(clippy::*)]`（除非另有 ADR 記錄）
- [ ] `commands/secret.rs` 無回傳 secret value 的 invoke handler
- [ ] HTTP client 統一收斂於 `services/server_client.rs`，commands 不直接呼叫 `reqwest`
- [ ] Domain service 不依賴 `tauri::*` types（可獨立測試）
