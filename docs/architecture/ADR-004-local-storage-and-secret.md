# ADR-004: Local Storage and Secret Management

Status: Proposed
Date: 2026-05-17
Owner: Project lead
Related: ADR-002, ADR-003, `docs/product/prd.md` §6.1 §14

## Context

桌面 app 需要在本機保存四類資料：

1. **Config** — server URL、plugin 開關、UI 偏好（人為可編輯）
2. **Mutable local state** — property cache、draft、queue、plugin status（需查詢、需 transaction）
3. **Large blob cache** — 匯入的 PDF/image、AI 輸出原文（檔案型，可 LRU 清掉）
4. **Secrets** — AI provider API key、server auth token（不能明文落地）

外加 log file 一類。

ADR-002 規定這些操作都走 Rust command surface，但沒定**存哪、用什麼**。本 ADR 鎖定。

## Decision

### 1. Config → TOML file

路徑：`$APPDATA/com.realestate-management.desktop/config.toml`

```toml
[server]
base_url = "http://192.168.1.6:8080"
health_check_interval_sec = 30
timeout_sec = 10

[plugins]
saydo_enabled = false
project_manager_enabled = false

[ui]
theme = "system"
dense_mode = true
locale = "zh-Hant-TW"

[telemetry]
local_log_level = "info"
```

- 讀寫由 `services/config.rs`
- Crate：`toml`、`serde`
- Atomic write：寫到 tmp file 再 rename，避免崩潰時殘檔
- 可由 UI 編輯 → 重啟生效（標 hot-reloadable 的欄位另議）

### 2. Mutable local state → SQLite

路徑：`$APPDATA/com.realestate-management.desktop/state.db`

Crate 選擇：**`sqlx` (sqlite feature, runtime-tokio-rustls)**
- 理由：async-friendly、compile-time SQL check、和 server 端若採 Postgres 共用 query 風格
- 替代方案：`rusqlite` 更輕，但 sync API 與 Tauri command async 不協調

Migration：**`refinery`**（embedded SQL files）
- migrations 放 `src-tauri/migrations/local/`
- App 啟動時 idempotent 跑

預期 table（v0）：

```sql
-- property summary cache（從 server 同步來的列表用）
CREATE TABLE property_summary_cache (
    id TEXT PRIMARY KEY,           -- UUID
    display_name TEXT NOT NULL,
    property_type TEXT NOT NULL,
    status TEXT NOT NULL,
    address_raw TEXT,
    last_synced_at INTEGER NOT NULL  -- unix timestamp
);

-- local-only drafts（未同步的 property）
CREATE TABLE property_drafts (
    draft_id TEXT PRIMARY KEY,
    payload BLOB NOT NULL,         -- serde_json::Value
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    sync_state TEXT NOT NULL       -- 'local_only' | 'syncing' | 'failed' | 'synced'
);

-- pending imports（server 連不上時的 queue）
CREATE TABLE pending_imports (
    import_id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    property_id TEXT,
    queued_at INTEGER NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT
);

-- plugin status
CREATE TABLE plugin_state (
    plugin_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL,      -- bool
    permissions BLOB NOT NULL,     -- json
    last_handshake_at INTEGER
);

-- run cache（最近觀察的 ProcessingRun，full state 仍以 server 為準）
CREATE TABLE run_status_cache (
    run_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    overall_status TEXT NOT NULL,
    last_polled_at INTEGER NOT NULL,
    snapshot BLOB NOT NULL         -- serde_json::Value of ProcessingRun
);
```

設計筆記：
- Cache table 永遠可被刪光 → 重新從 server 拉，所以**不存 canonical 資料**
- Draft table 才有 local-only canonical 風險，需有「上 server 前不能刪」保護
- `BLOB` 存 JSON 比 `TEXT` 略快且 SQLite 內無差別語意，但若要 query JSON path 改 `json`

### 3. Large blob cache → filesystem

路徑：`$APPDATA/com.realestate-management.desktop/cache/`

結構：

```
cache/
├── documents/
│   ├── ab/
│   │   └── ab12cd34...sha256.pdf
│   └── ...
└── ai-output/
    └── runs/
        └── <run_uuid>/
            ├── stage-1.json
            └── stage-2.json
```

- 檔名用 SHA-256 前綴分桶（避免單目錄太多檔）
- Cleanup：LRU + 總容量上限（預設 5 GB，可由 config 調），啟動時掃描清理
- Server 永遠是 source of truth；local cache 可隨時重抓

### 4. Secrets → OS keychain

Crate：**`keyring` (v3)**

- macOS：Keychain（service = `com.realestate-management.desktop`）
- Windows：Credential Manager
- Linux：Secret Service（依 desktop env 而定，可能需要 `libsecret` 套件）

Key naming convention：

```
service: "com.realestate-management.desktop"
account: "<scope>::<key>"

例：
"ai.openai::api_key"
"ai.anthropic::api_key"
"server.auth::token"
"plugin.saydo::shared_secret"
```

Rust API surface（給 `commands/secret.rs` 用）：

```rust
pub fn set_secret(scope: &str, key: &str, value: &str) -> Result<()>;
pub fn delete_secret(scope: &str, key: &str) -> Result<()>;
pub fn list_secret_keys(scope: &str) -> Result<Vec<String>>;  // values 不取
// 給 Rust services 用（不暴露到 WebView）：
pub(crate) fn read_secret(scope: &str, key: &str) -> Result<String>;
```

- **WebView 不可拿到 raw value**（ADR-002 已規定）
- AI provider key 由 `services/server_client.rs` 自己讀 keychain，注入 header
- secret 不寫進 log（structured logging 過濾 `*api_key*`、`*token*`、`*secret*` 欄位）

### 5. Logs → rolling file

路徑：`$APPDATA/com.realestate-management.desktop/logs/`

- Crate：`tracing` + `tracing-appender`（daily rotation）
- 保留 14 天
- Level 由 `config.toml [telemetry] local_log_level` 控制
- `RUST_LOG` env 可覆寫（dev 用）

## Directory Layout 全貌

```
$APPDATA/com.realestate-management.desktop/
├── config.toml
├── state.db
├── state.db-wal               (SQLite WAL)
├── state.db-shm
├── cache/
│   ├── documents/
│   └── ai-output/
└── logs/
    ├── app.2026-05-17.log
    └── app.2026-05-16.log
```

跨 OS 對應：
- macOS: `~/Library/Application Support/com.realestate-management.desktop/`
- Windows: `%APPDATA%/com.realestate-management.desktop/`
- Linux: `~/.local/share/com.realestate-management.desktop/`

由 Tauri `app_data_dir()` API 取得。

## Consequences

### 正面

- 四類資料分流，每類用最合適工具，故障域獨立
- Cache / log 可放心清掉，不會丟 canonical 資料
- Secret 用 OS 機制，跟使用者既有 password manager 整合

### 負面 / 成本

- Dependency 增加：`sqlx`, `refinery`, `keyring`, `toml`, `tracing-appender`
- Cross-platform Keychain 行為差異須測試（特別 Linux headless / SSH 場景）
- SQLite migration 在啟動 path 上，失敗要有 fallback（refuse to start with diagnostic）

## Alternatives Considered

| 方案 | 拒絕理由 |
|:--|:--|
| 全部用 `tauri-plugin-store`（JSON KV） | 沒 transaction、沒 index、queue 容易失序 |
| `tauri-plugin-stronghold` 管 secret | 過度複雜；OS keychain 已夠用且使用者熟悉 |
| `rusqlite` 取代 `sqlx` | Sync API 與 async command 不協調，需額外 spawn_blocking |
| 純 JSON files | 並發寫不安全；無 transaction；查詢效率差 |
| `sled` / `surrealdb` | Niche stack，社群與工具支援少於 SQLite |
| SQLCipher（SQLite + 加密） | 第一版不加；machine login + Keychain 已是合理基線。**列為未來選項** |

## Open Questions

1. **SQLite 是否加密？**
   建議：第一版**不加**。Local DB 只存 cache + draft + queue，canonical 在 server。若使用者要求加密，後續加 SQLCipher feature flag。

2. **Linux Secret Service 在 headless 環境怎麼辦？**
   建議：偵測到無 keychain 時 → fall back 到 prompt 使用者每次輸入（不寫盤），並在 UI 顯示「secret 未持久化」狀態。Phase 2 確認。

3. **macOS Keychain access prompt 體驗**
   第一次寫入需簽 app + entitlements，否則每次讀都會彈窗。等 scaffold 時設定 bundle id 與 signing。

4. **Cache 容量上限 / 清理策略**
   建議：預設 5 GB total，超過時依 LRU 清 `documents/`（AI output 較小，優先保留）。可由 config 調。

## Verification

scaffold 後：

```bash
cargo test --workspace --lib   # 含 sqlite migration up/down test
cargo test --test storage      # 整合測試：config 寫入/讀出、SQLite migration、keyring round-trip
```

Review checklist：
- [ ] `config.toml` 原子寫入（tmp + rename）
- [ ] SQLite migration 啟動時跑，失敗 abort with diagnostic
- [ ] Keychain key naming 統一在 `services/secret.rs` 常數
- [ ] Log 過濾器排除 secret 欄位
- [ ] `commands/secret.rs` 無回傳 value 的 invoke handler
