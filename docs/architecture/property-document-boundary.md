# Property / Document Data Boundary (Phase 2 baseline)

Status: Specification（非 ADR；落實 ADR-002 + ADR-004 + ADR-006 原則）
Date: 2026-05-18
Owner: Project lead
Related: ADR-002, ADR-004, ADR-006, `docs/architecture/data-model-v1.md`, `docs/product/prd.md` §10 §11, `.claude/rules/backend/rust-tauri.md`

## Purpose

ADR-002 規定了「什麼走 Tauri invoke / 什麼走 HTTP / 什麼 WebView 不可直接做」的原則；ADR-004 規定四類本機資料的存放方式；ADR-006 規定 canonical 走 Postgres + PostgREST。Phase 2 要在 property 與 document 兩個具體 domain 上開始實作，本文把上述三個 ADR 落到「每個 surface 走哪條」的具體 catalogue，避免實作時邊界飄移。

本文**不是新決策**。若實作中發現邊界不適用，先在這裡更新，再評估是否要寫新 ADR。

## Source-of-Truth 矩陣

| 資料類別 | Canonical 在哪 | 本機角色 | 失效時行為 |
|---|---|---|---|
| Property canonical record（confirmed evidence、status、kind、owner_actor） | Postgres（ADR-006） | `property_summary_cache` 列表用 / `run_status_cache` 用 / `property_drafts` 才有 local-only canonical | Server 不通 → 顯示 cache + degraded 標記；不可寫 confirmed |
| Property draft（尚未上 server） | 本機 SQLite `property_drafts` | Canonical | Server 通了 → 嘗試 sync；衝突進 `Conflict` state |
| Property AI extraction（非 confirmed 的 evidence） | Postgres `EvidenceValue<T>` JSONB | 不存本機 | 不可離線生成新 extraction |
| Document binary（PDF / 圖片原檔） | Server storage（MinIO / fs，依 `internal-server-plan.md`） | `cache/documents/<sha256>` LRU | Cache hit 顯示；無 cache 且 offline → 顯示 placeholder + retry |
| Document metadata（mime、size、associations、stage） | Postgres | 不獨立 cache，附屬於 `run_status_cache.snapshot` | 同 property canonical |
| Processing run（OCR + AI stage trace） | Postgres `processing_runs` + `stages` | `run_status_cache` 快取最近觀察 snapshot | Stage 還在跑 → 走 polling；offline → 凍結 snapshot + 顯示 stale |
| Pending document intake（已選檔、未送 server） | 本機 SQLite `pending_imports` + filesystem 暫存 | Canonical（直到送出） | 啟動時 resume queue |
| AI provider 原始輸出 | Server（與 run 綁定） | `cache/ai-output/runs/<run_uuid>/stage-*.json` | Cache hit 才顯示原文；不可離線生成 |

不變式：
1. **canonical = server**，本機 cache 隨時可刪、隨時重抓。
2. **local-only canonical 只在 drafts / pending_imports 兩處**，這兩張表的刪除需保護（draft 未 sync 前不可丟；pending intake 必須送達或明確取消）。
3. AI 從未 `confirmed` 任何 evidence；不論 cache 或 server，AI 寫入只能改 `ai_extracted` / `ai_stage_id`。

## Surface Routing — Property

走 **Tauri invoke**（Rust command surface，無 HTTP）：

| Command | 用途 | 對應檔案計畫 |
|---|---|---|
| `list_property_drafts` | 列本機 draft（尚未上 server） | `commands/property.rs` + `services/property_local.rs` |
| `save_property_draft` | 新增 / 更新 local-only draft | 同上 |
| `discard_property_draft` | 確認刪除 local-only draft（需 UI 二次確認） | 同上 |
| `queue_property_sync(draft_id)` | 把 draft 排入 sync queue（背景送 server） | 同上 |
| `peek_property_cache_age(property_id)` | 回傳 cache snapshot 與 `last_synced_at`，供 UI degraded badge | `commands/property.rs` |
| `pick_document_files()` | 開檔案 dialog（Tauri `dialog` allowlist） | `commands/document.rs` |

走 **HTTP → server**（透過 `services/server_client.rs`，PostgREST or Rust axum 看 endpoint）：

| Path | 方法 | 用途 | 不變式 |
|---|---|---|---|
| `/api/properties?select=...` | GET | 列表 / search / filter | anon role + SELECT only |
| `/api/properties?id=eq.<uuid>` | GET | 取單筆 canonical | 同上 |
| `/api/rpc/save_property` | POST | 寫入 confirmed（包 RLS check） | service_role via Rust server-side proxy |
| `/api/rpc/confirm_property_field` | POST | 把某 evidence field 推進 `Confirmed` | 須附人類確認上下文（`actor_id`、`confirmed_at`） |
| `/api/audit_events?property_id=eq.<uuid>` | GET | 該物件 audit log | append-only |

WebView **不可直接打**：
- 任何 PostgREST endpoint（必走 Rust HTTP client，集中加 timeout / retry / log filter）
- 任何 AI provider URL（必走 server 端 routing）
- 任何 fs / shell（ADR-002 已禁）

## Surface Routing — Document

走 **Tauri invoke**：

| Command | 用途 |
|---|---|
| `pick_document_files()` | 開檔案 dialog；只回 path，不讀內容 |
| `enqueue_document_import(file_path, property_id?)` | 計算 SHA-256，複製到 `cache/documents/<sha256>`，寫入 `pending_imports` |
| `list_pending_imports()` | UI 顯示 queue |
| `retry_pending_import(import_id)` | 觸發 sync worker 重試 |
| `cancel_pending_import(import_id)` | 取消（需 UI 二次確認） |
| `read_cached_document_blob(sha256)` | 給 PDF viewer 讀 cache 內容（只回 bytes，不上傳） |

走 **HTTP → server**：

| Path | 方法 | 用途 |
|---|---|---|
| `POST /api/documents` | 上傳 binary + metadata | multipart；server 落地後回 `document_id` |
| `GET /api/documents?id=eq.<uuid>` | 取 metadata | |
| `GET /api/documents/<uuid>/blob` | 下載 binary（cache miss 時） | server 可重導向 storage |
| `POST /api/rpc/start_processing_run` | 啟動 OCR + AI pipeline | 回 `run_id` |
| `GET /api/processing_runs?id=eq.<run_id>&select=*,stages(*)` | Poll stage 狀態 | UI 用 |

Polling interval 統一從 `services/server_client.rs` 走，UI 不自行設 interval（避免每個 page 各自定）。

## Failure Mode 對照

| 情境 | UI 顯示 | 可操作 | 不可操作 |
|---|---|---|---|
| Server unreachable | Status badge: `offline`；cache 列表加 `stale: <age>` | 看 cache、編 draft、queue 新 import | confirm evidence、view 未 cache 的 doc、start AI run |
| Server degraded（部分 service down） | Badge: `degraded`；點開看哪個 service down | 同 healthy minus 失效部分 | 對應 down 服務的功能 |
| AI provider failure | Run 卡在某 stage，state = `Failed` with `failure_reason` | Retry stage、改 provider、人工確認跳過 stage | 自動 fallback 補假資料（嚴禁） |
| OCR failure | 同上 | Retry、改檔案 | 直接 confirm evidence（必須 humanEdited 後才能 confirm） |
| Draft sync conflict | Draft state = `Conflict`，並列 server 與 local 兩版 | 選一邊、merge、捨棄 | 自動 overwrite（嚴禁） |
| Cache 容量超過上限 | LRU 清 `cache/documents/`，AI output 保留 | 看 server canonical 即可 | — |

## v0 → v1 演進

v0（Phase 2 第一刀）：
- 只實作 Property list + detail（read），走 PostgREST GET
- Document import 走 Tauri command（pick + enqueue + cache）
- Processing run 用 polling 顯示 stage
- 不開 `save_property` / `confirm_property_field`（避免在 RLS / service_role 設計穩定前先實作）

v1（Phase 2 中後段）：
- 開 Rust axum `/api/rpc/save_property`，走 service_role 寫 server
- Draft → sync flow 全鏈接通
- Audit event 寫入並回讀
- Conflict resolution UI

不在 Phase 2 範圍：
- 多 user / RLS by `auth.uid()` → 待 ADR-007（auth）
- PostGIS query → 待 ADR-008
- Plugin contract 把 property 推給 Project-Manager / SayDo → 另排

## Open Questions

1. **PostgREST 直接給 WebView 還是只給 Rust？**
   建議：v0 連 GET 都走 Rust，理由是集中 secret（auth token）、log 過濾、timeout 控制。代價是多一層 hop。若效能成為問題，再讓 list/search 等 read-only endpoint 直接從 WebView 打 PostgREST。
2. **Cache invalidation 策略**
   建議：寫入成功後 server 回 `last_modified`，本機 cache 比對；不寫 push notification。
3. **PDF viewer 用 cache 還是 server stream？**
   建議：先 cache（簡單、可離線）；blob 超過 50MB 改用 stream。閾值由 config 控。
4. **Pending intake 的暫存檔何時清？**
   建議：成功送 server 後立即刪；失敗 retry 上限後仍失敗保留並標 `last_error`，由 UI 引導人工處理。

## Verification

Phase 2 完成 v0 後檢核：

- [ ] WebView 程式碼沒有任何 `fetch('/api/...')` 直連 server（全部走 Rust invoke 或 `services/server_client.rs`）
- [ ] `commands/property.rs` 與 `commands/document.rs` 對應上方表格 surface
- [ ] `services/server_client.rs` 是唯一 HTTP 出口，含 timeout / retry / auth header
- [ ] `property_summary_cache` / `run_status_cache` 可整表 truncate 後 app 仍能從 server 重建
- [ ] `property_drafts` / `pending_imports` 整表 truncate 會丟使用者資料 → 需有 UI 警告與不可誤刪保護
- [ ] Degraded / offline badge 在所有 read 列表上正確顯示
