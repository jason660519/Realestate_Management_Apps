# Dev Log: Phase 2 Read-Only Property Surface

Date: 2026-05-18

延續 `dev-phase2-foundations-2026-05-18.md`，本場開 Phase 2 v0 第一刀：
集中 HTTP 出口、property read-only surface 上線（backend only）。

## Completed

### 1. `services/server_client.rs`：集中 HTTP 出口

- 新增 `ServerClient` 結構，唯一持有 `reqwest::Client`，所有外送請求皆從此處出去。
- `ServerClient::from_config()` 回 `Result<Option<Self>, AppError>`：空 base URL 回 `None`，呼叫端據此回 `not_configured`，不假裝送出失敗請求。
- `get_json::<T>(path)`：generic GET + JSON deserialize；非 2xx 與 transport 錯誤都包成 `AppError::ServerUnreachable`，訊息帶 URL 與 status code。
- `check_health()`：搬 `commands/server.rs` 的 reqwest 邏輯進來，回 `ServerHealth`（ok / degraded / offline）。
- `check_health_for_config(config)`：頂層方便入口；先處理 `not_configured` case，否則建 client 跑 health check。

### 2. `services/property_service.rs`：read-only list

- `list_property_summaries(config) -> Result<Vec<PropertySummary>, AppError>`。
- v0 走 PostgREST：`GET /api/properties?select=id,display_name,kind,status,address_raw,updated_at&order=updated_at.desc&limit=100`。
- 空 base URL 回 `Ok(vec![])`，UI 顯示 not configured 不需 error toast。
- `SUMMARY_SELECT` 常數與 `PropertySummary` model 並排維護，欄位增刪需同步。

### 3. `models.rs`：`PropertySummary` 與兩個 enum

- 新增 `PropertyStatus { Draft, Active, Pending, Archived, Unknown }`、`PropertyKind { Sale, Rental, LandOnly, Commercial, Unknown }`。
- 兩者都帶 `#[serde(other)] Unknown` fallback：legacy migration 帶進來的 enum 值不會炸 deserialize，UI 可以保留行顯示「未知類型」。
- `PropertySummary` 用 PostgREST 預設 snake_case；frontend 接到的 wrapper 自帶 camelCase 轉換（之後 invoke 一次性處理）。

### 4. `commands/property.rs`：薄殼 invoke handler

- `#[tauri::command] list_property_summaries(state)` 取 `AppConfig` 後 delegate 到 service。
- 不做格式化、過濾、cache — UI 自行處理 stale / cache，呼應 `property-document-boundary.md` 的 source-of-truth 矩陣。

### 5. `commands/server.rs` 改薄殼

- 從 70+ 行的 reqwest + match 邏輯瘦身為 8 行：取 config → delegate 給 `server_client::check_health_for_config`。
- Health check 的 unit tests 移到 `services/server_client.rs`，與 generic `get_json` 測試集中一處（TcpListener mock pattern 同一份）。

### 6. `lib.rs` 註冊新 command

- `invoke_handler` 新增 `list_property_summaries`。
- `commands/mod.rs` re-export。

## Verification

- `cargo fmt --check`：通過。
- `cargo check`：通過，無 warning。
- `cargo clippy --all-targets -- -D warnings`：通過。
- `cargo test`：13 passed / 13（含 3 個 server_client 新測試、3 個 property_service 新測試、其餘既有不變）。
- `npm run typecheck`：通過。
- `npm run test`：14 passed / 14（frontend 未動，回歸測試確保）。

## Technical Notes

- `ServerClient::from_config` 回 `Option<Self>` 而非 panic / Err：把 `not_configured` 變成第一級語意，避免每個 caller 各自判斷 base URL。Health check 的 not_configured / property_service 的 empty list 都從這裡分流。
- `PropertyStatus` / `PropertyKind` 用 `#[serde(other)]` 而非 `Unknown(String)`：前者避免 untyped string 流回 UI，後者導致 enum match 永遠要兜底；evidence-first 設計裡，未知值仍應該以分類存在，而不是當作 free-form。代價：丟失原始字串，需要時補欄位。
- HTTP path 用 `format!("{}{}", base, path)` 串接，避免 reqwest 的 url join 對 `?query` 行為的歧義；caller 必須自己帶 `/` 前綴。文件已說明。
- 每個 invoke 重建 `ServerClient`：reqwest::Client 內部已 connection pool，再加上 invoke 不會高頻，重建成本可接受。若實測有問題，再做 OnceCell 快取。
- 中文測試資料：用 raw string `r#"..."#.as_bytes()` 不能用 byte string `br#"..."#`（Rust 限制 byte literal 為 ASCII）。

## Open Questions

1. **PostgREST 認證 header**：v0 anon role 不帶 token，但 RLS 在 ADR-006 預留 service_role 寫入路徑。等 server 部署到 192.168.1.6 時補 `Authorization: Bearer <PGRST_JWT>`。
2. **Cache 寫入時機**：`ServerClient::get_json` 目前不寫 SQLite cache（ADR-004 規劃的 `property_summary_cache`）。Phase 2 中段補：成功 fetch 後寫 cache、`last_synced_at`；offline 時 UI 從 cache 讀並標 stale。
3. **Property kind 對應 `data-model-v1.md`**：data-model-v1 列了 `Sale | Rental | LandOnly | Commercial`，本次 enum 完全對齊。若 i18n / AU 階段補新 kind，需同步更新 `[#serde(other)] Unknown` 之外的 variants。

## Next Priority

- Frontend Properties 頁串接 `list_property_summaries`：需要先建 `src/test/render.tsx`（Mantine wrapper）與 `src/test/mockTauri.ts`，再做 component test。
- ADR-004 規劃的 SQLite cache 第一張表 `property_summary_cache` 動工，搭配 `ServerClient::get_json` 寫 cache。
- Phase 2 write path 規劃：`save_property` 走 Rust axum service_role（非 PostgREST 直寫），需新 ADR 或追補 ADR-006 的 service_role 部分。
- 處理三筆 untracked（`.project-manager.json`、`docs/dev-logs/.gitkeep`、`docs/features/.gitkeep`）— 第三 session 仍未動，建議列入 `.gitignore` 或刪除。
