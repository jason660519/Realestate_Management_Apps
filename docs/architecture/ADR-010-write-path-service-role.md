# ADR-010: Write Path via Rust axum + service_role

Status: Proposed
Date: 2026-05-18
Owner: Project lead
Related: ADR-002, ADR-006, `docs/architecture/property-document-boundary.md`, `docs/architecture/data-model-v1.md`

## Context

ADR-006 §「Reasoning §4 Auth 延後」與「Consequences 負面 §2」已定 v0 PostgREST 的 `anon` 只給 SELECT，**寫入由 Rust server-side 走 service_role**。Phase 2 read path 已上線（`list_property_summaries` 透過 `ServerClient::get_json` 直打 PostgREST anon），下一步要把 write surface 落地。`property-document-boundary.md` 列了下列尚未實作的 RPC：

- `POST /api/rpc/save_property`
- `POST /api/rpc/confirm_property_field`

兩者都涉及 evidence-backed 欄位、RLS 與 audit 追蹤，**不適合**讓 PostgREST 直接收。本 ADR 鎖定 write path 形狀。

## Decision

### 1. Write path 走 Rust axum service，**不**走 PostgREST RPC

新增 server 端 component：

```
internal-server-plan.md baseline + 新增：
- realestate-api（Rust axum on port 8081，與 postgrest 同 docker compose）
```

理由：
- PostgREST RPC 對複雜 validation / multi-table transaction（`property` + `audit_events` 同步寫）支援有限
- service_role token 不可暴露到 PostgREST anon endpoint；axum 集中持有
- evidence-first 規則（status invariants、AI 不可寫 confirmed）需要 typed enforcement，PostgREST 只能做 row-level check
- 將來加 GoTrue auth 後 RLS by `auth.uid()` 可繼承到 axum（axum 解 JWT → 設 postgres session role）

不採方案：
- PostgREST RPC + Postgres function：trigger / function 寫滿太脆弱
- pg-graphql：同問題加上複雜性增加
- 直接讓桌面 app 用 service_role token 打 PostgREST：service_role 不可流到 endpoint 外（無 RLS 等同 admin）

### 2. Reverse-proxy 路由

對 desktop client 與 boundary doc 對齊：

| Path prefix | 目的地 | 角色 |
|---|---|---|
| `/api/properties` (GET 只讀) | `postgrest:3000` | anon SELECT only |
| `/api/documents` (GET 只讀) | `postgrest:3000` | anon SELECT only |
| `/api/processing_runs` (GET 只讀) | `postgrest:3000` | anon SELECT only |
| `/api/audit_events` (GET 只讀) | `postgrest:3000` | anon SELECT only |
| `/api/rpc/save_property` | `realestate-api:8081` | service_role + Rust validation |
| `/api/rpc/confirm_property_field` | `realestate-api:8081` | service_role + Rust validation |
| `/api/rpc/start_processing_run` | `realestate-api:8081` | service_role + AI provider routing |
| `/api/documents` (POST 上傳) | `realestate-api:8081` | service_role + storage 寫入 |

reverse-proxy（待 `internal-server-plan.md` 補）用 path prefix routing，**不**讓 desktop client 知道後面是 PostgREST 還是 axum。Desktop 永遠打 `/api/...`，內部由 proxy 分流。

### 3. axum service 結構

```
realestate-api/
├── Cargo.toml
├── src/
│   ├── main.rs              # HTTP server bootstrap, axum router
│   ├── auth.rs              # service_role token loader, future JWT verify
│   ├── db.rs                # sqlx PgPool（service_role connection）
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── property.rs      # save_property / confirm_property_field
│   │   ├── document.rs      # upload + multipart
│   │   └── processing.rs    # start_processing_run, stage progression
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── property.rs      # validation + evidence invariants
│   │   └── audit.rs         # AuditEvent 建構
│   └── errors.rs            # API error → HTTP status mapping
└── tests/
    ├── property_save.rs     # integration: 寫一筆 + audit_event 同 transaction
    └── ...
```

放 server 上獨立 binary（**不**在 `src-tauri/` 內）。Desktop app 完全不依賴此 crate；兩者用 HTTP 解耦。

依賴：
- `axum = "0.8"`（或同期 stable）
- `sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono", "macros"] }`
- `tower = "0.5"` + `tower-http`（CORS / trace / timeout middleware）
- `serde` / `serde_json`
- `thiserror` for domain errors

### 4. service_role token 管理

- token 存 server `.env.local`：`PGRST_SERVICE_ROLE_KEY=<long random>`
- axum 啟動時讀 env，**不**寫 log
- sqlx connection string：`postgresql://service_role_user:<password>@postgres:5432/realestate`（postgres user `service_role_user` 在 schema 設好 `BYPASS RLS`）
- 重要：**desktop app 不持有 service_role token**。Desktop 對 axum 用 anon flow（v0），axum 自行決定能否寫

v1（多 user）：desktop app 持使用者 JWT（GoTrue 簽發）→ axum 解 JWT → 取使用者 id 寫入 audit_event、且依 RLS policy 決定是否能寫。

### 5. Evidence-first 邏輯落在 axum domain 層

`save_property`：

```rust
pub async fn save_property(
    pool: &PgPool,
    actor: ActorId,
    payload: SavePropertyRequest,
) -> Result<Property, ApiError> {
    let mut tx = pool.begin().await?;
    
    // 1. 取目前狀態（若是更新）
    // 2. validate: confirmed-only fields 不可被 AI payload 推進
    //    - 對每個 EvidenceValue，若 incoming has `confirmed.Some` 必須有 actor + reason
    //    - 若 incoming 只有 ai_extracted，status 限 AiDone，不可碰 confirmed
    // 3. UPDATE / INSERT property
    // 4. 對每個 evidence 變更欄位寫一筆 audit_event（field-level）
    // 5. commit
}
```

audit_event 寫入：

```sql
INSERT INTO audit_events (
    id, property_id, document_id, field_path, before_status, after_status,
    actor_id, action, reason, occurred_at
) VALUES (...);
```

`field_path` 對齊 `EvidenceValue<T>` 在 JSONB 內路徑（`$.address`、`$.building_area`、…）。

### 6. Error contract

axum handler 回 `ApiError` → HTTP status + JSON body：

```json
{
  "kind": "validation",
  "message": "human-readable",
  "stage": "confirm | save | audit",
  "field": "address (optional)"
}
```

Desktop `ServerClient::get_json` 反序列化失敗或 4xx → 桌面 UI 顯示 `What failed / preserved / next action`（呼應 design doc §Error states）。Status code 對應：

| Status | Kind | UI 文案 |
|---|---|---|
| 400 | `validation` | 「{field} 不通過：{message}」 |
| 401 | `unauthenticated` | 「請重新登入」（v1+ 才會出現） |
| 403 | `forbidden` | 「此欄位需要更高權限確認」 |
| 409 | `conflict` | 「資料已被他人更新，請重抓再試」 |
| 422 | `evidence_invariant` | 「AI 不可直接寫 confirmed 欄位」 |
| 500 | `internal` | 「server 端錯誤，操作未生效」 |

### 7. 與 desktop 的整合點

Desktop 端新增 `commands/property.rs`：

```rust
#[tauri::command]
pub async fn save_property(
    state: State<'_, AppState>,
    payload: SavePropertyPayload,
) -> Result<Property, AppError> {
    let config = state.config()?;
    property_service::save_property(&config, payload).await
}
```

`services/property_service.rs` 加：

```rust
pub async fn save_property(
    config: &AppConfig,
    payload: SavePropertyPayload,
) -> Result<Property, AppError> {
    let Some(client) = ServerClient::from_config(config)? else {
        return Err(AppError::InvalidInput {
            message: "Server URL must be configured before saving".into(),
        });
    };
    client.post_json("/api/rpc/save_property", &payload).await
}
```

ServerClient 需要新增 `post_json::<TIn, TOut>(path, body)`，待實作時補。

## Consequences

### 正面

- v0 read 走 PostgREST 直連（快、無 boilerplate）；v0 write 走 typed axum（evidence invariant + audit 連動）
- service_role token 收斂在 axum 一處，桌面端 never 持有
- 路徑統一 `/api/...`，desktop 不需知道後面是誰
- 未來 GoTrue auth 起來時，只動 axum，desktop 不必改

### 負面 / 成本

- 多一個 server crate 要維護（CI / deploy / dependency）
- v0 → v1 axum service 從零寫，需要 1-2 週工時（property save / confirm_field / document upload / processing run）
- reverse-proxy 規則複雜化（多一條 path prefix）

### 風險

- **路徑漂移**：desktop 寫死 `/api/rpc/save_property`，server 端若改名會破。Mitigation：axum 與 desktop 用 shared OpenAPI（或手寫 client SDK）；先用 contract test 鎖住路徑
- **service_role 外洩**：env 檔 chmod 600；axum 程序 dump core 風險（disable core dump in systemd unit）
- **transaction 卡死**：write path 內 multi-table update 不可超過 5 秒，否則 timeout；UI 顯示「pending」並 retry

## Open Questions

1. **axum crate 放哪個 repo？** 建議：獨立 repo `realestate-management-api` on internal git server（或 GitHub private），由 `docs/deployment/internal-server-plan.md` 紀錄。**不**塞進這個 desktop repo，避免 build 時間膨脹
2. **共用 type definitions（Property、EvidenceValue<T>）放哪？** 三個選項：
   - axum 端定義，desktop 用 OpenAPI 生 TypeScript / Rust client（heavy 工具鏈）
   - 共用 `realestate-shared` crate（兩端都 vendored）— **建議**
   - 兩端各自 hand-write（最快但漂移風險最大）
   選擇待 axum repo 啟動時拍板
3. **Conflict resolution（409）的 UI 流程**：sole user 暫不會撞，留 ADR-007 auth 後再規劃
4. **AI provider routing 也要放 axum？** PRD §6.1「外部 AI API 不從 WebView 也不從 Rust 桌面端打」→ 是。`/api/rpc/start_processing_run` 內 axum 自行 routing
5. **migration 路徑**：與 PostgREST 共用 `infra/migrations/`（ADR-006 §「Schema Migration Strategy」），不額外開 axum migration

## Migration Path (若退場)

若日後決定 service_role 邏輯回 PostgREST function：
1. 把 axum handler 內 SQL 搬成 `CREATE FUNCTION ... SECURITY DEFINER`
2. PostgREST 自動暴露 `/rpc/save_property`
3. reverse-proxy `/api/rpc/save_property` 改指 `postgrest`
4. Desktop 不需改

但 evidence-first invariant 在 function 內檢查辛苦，且 audit_event 寫入要靠 trigger，整體不推薦。

## Verification

axum service 上線後：

- [ ] `curl -X POST http://192.168.1.6:8080/api/rpc/save_property -d @sample.json` 回 200 + Property JSON
- [ ] `audit_events` 表多一筆對應變更
- [ ] PostgREST `anon` 無法呼叫 `/api/rpc/save_property`（應該 404，因為 proxy 不轉）
- [ ] Desktop `invoke('save_property', { payload })` 與 server response 對齊
- [ ] axum integration test 覆蓋 evidence invariant（AI 不可寫 confirmed）

Desktop 端先行交付：

- [ ] `ServerClient::post_json` 實作 + unit test（mock 200 / 4xx / 5xx）
- [ ] `commands/property.rs save_property` 薄殼接好 + unit test
- [ ] frontend save UI 等 PRD Phase 2 中段 design 完成後實作（不在本 ADR）
