# Dev Log: SQLite Cache + Write Path ADR + Bundle Refresh

Date: 2026-05-18

第四場（同日累計第六份 dev log）。本場完成 untracked 清理、ADR-010
（write path）、unsigned bundle rebuild，與 Phase 2 SQLite cache 第一張
表上線（含 backend + frontend 整合）。

## Completed

### A. Untracked 清理

- `.project-manager.json` 加進 `.gitignore`（外部工具狀態，與 desktop app 無關）。
- 刪除 `docs/dev-logs/.gitkeep`、`docs/features/.gitkeep` 空殼（路徑與規則 `docs/project-process/dev-logs/` 衝突）。
- working tree 完全乾淨。

### B. ADR-010 Write Path via Rust axum + service_role

- 新增 `docs/architecture/ADR-010-write-path-service-role.md`。
- 決策：save_property / confirm_property_field / start_processing_run 走獨立 axum service（**不**塞進 desktop repo），透過 reverse-proxy 分流；PostgREST anon 只給 SELECT；evidence-first invariant 由 axum domain 層 typed enforce；audit_events 寫入與 property update 同 transaction。
- 列出 path → service 對照、service_role token 持有規則（desktop 從不持有）、ApiError → HTTP status → UI 文案契約、退場路徑。
- Open Questions 列了 5 點待 axum repo 啟動時拍板（包含 shared types 策略）。

### C. Unsigned bundle 重 build

- `npm run tauri:build` 重打：release 編譯 14s（增量），DMG 4.1MB aarch64。
- 新 bundle 含 Properties 頁五態 view。
- 路徑：`src-tauri/target/release/bundle/{macos,dmg}/`。

#### 驗收 Checklist（請手動跑）

1. Mount `Realestate Management Apps_0.1.0_aarch64.dmg`，把 app 拖到 Applications。
2. 首次開啟需 right-click → Open（unsigned，依 ADR-009 v0 範圍）。
3. 確認 icon 在 Dock、Finder、About dialog 正確。
4. Workbench / Documents / AI Review / Tasks / Integrations / Settings 七個 nav rail 都可點。
5. **Properties 頁**：
   - 若 server URL 已設且可達 → 看到 Mantine Table（或 Not configured / Empty 空狀態）。
   - 拔網路（或關 server）後重 Refresh → 應出現黃色「Showing cached property list」banner + Last synced timestamp。
   - 復原網路重 Refresh → banner 消失、回到 live。
6. Settings 頁的 storage diagnostics 區應顯示 `~/Library/Application Support/com.realestate-management.desktop/`，且 `state.db`（SQLite）會在此目錄出現。

### D. SQLite cache `property_summary_cache` 上線

#### Backend

- Cargo.toml：加 `sqlx 0.8`（sqlite + runtime-tokio-rustls + chrono + macros + migrate），dev-dependencies 加 `tokio 1`（macros + rt-multi-thread）給 `#[tokio::test]`。
- 新增 `src-tauri/migrations/20260518000000_create_property_summary_cache.sql`：本機 cache 表，含 `last_synced_at` index。**migration 系統**：用 sqlx 內建 `migrate!()` macro，**不**用 ADR-004 提的 refinery（理由：sqlx 已 pull-in、embedded migrations 已內建、減少依賴）。ADR-004 §「Migration：refinery」段以此 commit 為實作準。
- `services/local_db.rs`：`open(app_data_dir)` 開 pool + 跑 migration + WAL 模式；`open_in_memory()` test helper。
- `services/property_cache.rs`：`replace_summaries` 用 transaction（DELETE + bulk INSERT），`read_summaries` 回 `CachedSummaries { rows, last_synced_at }`。enum 用 manual `serde_kind` / `deserialize_kind` 避免 sqlx 對 sqlx `Type` impl 強耦合。
- `services/property_service.rs`：改為 **write-through cache + stale fallback**。Live fetch 成功時寫 cache、回 `source: 'live'`；失敗（unreachable / 5xx / parse error / blank base URL）讀 cache 回 `source: 'cache'` 並帶 `error`；無 cache 時回 `source: 'empty'`。
- `models.rs`：新增 `PropertySource` enum 與 `PropertySummariesResult` wrapper（camelCase 出 frontend）。
- `commands/property.rs` 改用新 return type。
- `state.rs`：`AppState` 加 `local_db: SqlitePool` field；test 改 async（`#[tokio::test]`）。
- `lib.rs` setup：用 `tauri::async_runtime::block_on` 跑 `local_db::open`，把 pool 注入 AppState。
- `errors.rs` 新增 `AppError::LocalDb`。

#### Frontend

- `api/tauri.ts`：新 `PropertySource` union + `PropertySummariesResult` type，`listPropertySummaries` 回 wrapper（含 preview-mode 假資料）。
- `routes/properties.tsx`：
  - 接 wrapper：empty/live/cache 三條分支。
  - 新 `StaleCacheBanner`：source=cache 時顯示黃色 `IconCloudOff` Alert，列 last synced 與 reason。
  - 文案：保留 evidence-first 規則「confirmation actions and saves are disabled until the server is back」。
- `routes/properties.test.tsx`：擴成 6 個 case，新增 stale cache banner 顯示驗證；其餘 5 個更新為 wrapper shape。

## Verification

- `cargo fmt --check`：通過。
- `cargo clippy --all-targets -- -D warnings`：通過。
- `cargo test`：17 passed / 17（property_cache 4 + property_service 3 + 既有 10）。
- `npm run typecheck`：通過。
- `npm run test`：20 passed / 20（既有 14 + properties.test.tsx 6）。
- `npm run build`：通過。
- `npm run tauri:build`：成功（前段 B 的 build；SQLite cache 改動在 build 之前的版本，**新版尚未重 build**）。

> **注意**：D 段 SQLite cache 改動在 C 段 build 之後才完成。要看 cache 在實機跑，需再跑一次 `npm run tauri:build`，否則 mount 的 .dmg 是 unsigned 版本但**沒有** cache 行為。

## Technical Notes

- sqlx 內建 `migrate!()`：編譯時讀 `./migrations/` 目錄、生成 embedded migrator；migration 命名遵循 `YYYYMMDDHHMMSS_description.sql` 對齊專案 general rule §「Migration」。
- WAL mode：SQLite 預設 rollback journal 在多進程 / 多 thread 寫入時鎖太緊；WAL 對 desktop app（單進程多 connection）更友善。
- Cache 寫入失敗**不**讓 live response 失敗：log via stderr + 回 server data。理由：「成功 fetch 但 cache 寫不進」比「成功 fetch 但 caller 看不到」更可挽回。
- `replace_summaries` 用 DELETE + INSERT，不用 `INSERT ... ON CONFLICT`：summary projection 本來就是 server-derived snapshot，整批換比 row-by-row upsert 簡單且符合語意。
- `PropertyKind` / `PropertyStatus` 在 cache 表用 string 儲存而非 INTEGER：對未來 schema migration / 直接看 db 內容更友好；代價是欄位寬度多幾 byte，桌面 single-user 場景可忽略。
- `last_synced_at` 用 `MAX()` 於 read 端聚合而非單列存：所有 row 來自同一批次寫入，但讀回時 `MAX()` 是 idempotent fallback 萬一未來變成 partial update。
- Tauri setup 內 `block_on(local_db::open(...))`：setup 是 sync callback，但 sqlx 全 async；block_on 是 Tauri 提供的 startup-time async bridge，等同其他長壽 service 啟動 pattern。

## Open Questions

1. **Cache 容量上限**：ADR-004 §3 提 5GB total 給 `cache/documents/`；本次只動 SQLite schema 沒上限。預期 summary cache 不會大（每筆 < 1KB），暫不加 LRU。
2. **Migration 順序保證**：sqlx `migrate!()` 依檔名字典序跑；timestamp 命名足夠。要小心同一秒兩筆 migration（並行開發時），用 14 位數字而非 12 位 timestamp 給 sub-second 區分。
3. **Cache invalidation 觸發點**：目前只有「成功 fetch 後 replace」。Phase 2 寫入路徑（save_property）會新增另一觸發點：寫入成功後要 cache evict / refresh。等 axum service 上線時補。
4. **`/dev` console log（eprintln!）**：cache write 失敗目前用 stderr。Production app 看不到 stderr 除非 launchd / Tauri log plugin。等 tracing 進來統一改 `tracing::warn!`。

## Next Priority

- 重 build `.dmg` 一次（含 SQLite cache 行為），請使用者重新驗收。
- 視覺驗收後若 OK，可開始 Phase 2 寫入路徑：
  - 等 axum service repo 起來；同時可先在 desktop 加 `ServerClient::post_json` + `commands/property::save_property` 薄殼（unit test mock 200/4xx/5xx）。
- 補 React component test 第二輪：Settings smoke test、ShellLayout active state 等（helper 已成熟）。
- ADR-010 §「Open Questions §2 共用 types」決議 → 啟動 axum repo 時定案。
