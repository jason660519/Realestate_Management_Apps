# ADR-006: Database — Supabase Self-Hosted (Lean Stack)

Status: Proposed
Date: 2026-05-17
Owner: Project lead
Related: ADR-002, `docs/architecture/frontend-and-db-trade-offs.md`, `docs/deployment/internal-server-plan.md`, `docs/migration/legacy-schema-audit.md`

## Context

DB 選擇是 Phase 0 最後一個 blocker。`frontend-and-db-trade-offs.md` 已列出三種方案：Supabase Cloud / Supabase Self-Hosted / Plain Postgres + Custom API。

驅動條件：

- PRD §6.2：重型服務在內網 server `rick@192.168.1.6`，排除 Cloud
- Audit 顯示 legacy 是 Supabase + 188 個 migration + 170 個 table + 廣泛 RLS
- 桌面 app v0 是 single-user / local operator，不需要 Supabase auth 的多 user 功能
- 不需要 realtime（單機）、不需要 edge functions（Rust server 替代）、不需要 studio container（用 DBeaver / TablePlus）

## Decision

採用 **Supabase Self-Hosted 精簡 stack on `192.168.1.6`**。

部署兩個核心容器：

| 容器 | 角色 |
|:--|:--|
| `postgres:17` | Canonical 資料庫（含 PostGIS extension，雖然 v1 暫不採用 geom）|
| `postgrest:v12` | 自動產生的 REST API |

不部署：
- Supabase Studio（用桌面 DB 工具）
- Realtime（v0 單機不需）
- Edge Functions（Rust server-side 取代）
- GoTrue / Auth（v0 暫不啟用；多 user 時再開）
- Storage（用 MinIO 或 fs + nginx，由 internal-server-plan 規範）

## Reasoning

1. **Legacy schema 平滑遷移**
   Audit §5 顯示大量 table 是「direct map」或「reshape」，且 RLS / `auth.uid()` 慣例可在新版選擇性繼承。Supabase → Supabase（精簡）保留 schema 工具鏈相容性，省掉重寫 PostgREST endpoint。

2. **PostgREST 即拿即用**
   Phase 2 property / document CRUD 可直接走 PostgREST，不必先寫 typed API。Server-side Rust 層在以下情況才補：
   - AI provider routing（不適合放 PostgREST）
   - OCR / document processing 編排（multi-step、long-running）
   - 跨表 evidence 聚合 query（PostgREST embed 寫不下時）

3. **精簡 stack 維運接近純 postgres**
   2 個 container 比完整 Supabase stack（8+ container）易管理。升級鏈短：postgres patch + PostgREST patch 各自獨立。

4. **Auth 延後**
   v0 桌面是 local operator，內網信任。等 Phase 3+ 多 user / 多裝置才開 GoTrue。屆時 RLS policy 可以複用 legacy 模式（`auth.uid()`）。

## Container Spec

對齊 `internal-server-plan.md` §3 / §4：

### `postgres`

```yaml
image: postgres:17
restart: unless-stopped
environment:
  POSTGRES_DB: realestate
  POSTGRES_USER: ${PG_USER}                # from .env.local
  POSTGRES_PASSWORD: ${PG_PASSWORD}
volumes:
  - pg_data:/var/lib/postgresql/data
ports: []                                  # 不對 host 暴露
networks: [internal]
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U ${PG_USER}"]
  interval: 10s
extensions:
  - uuid-ossp
  - pgcrypto
  - postgis        # install 但 v1 暫不用 geom
```

### `postgrest`

```yaml
image: postgrest/postgrest:v12
restart: unless-stopped
environment:
  PGRST_DB_URI: postgres://${PG_USER}:${PG_PASSWORD}@postgres:5432/realestate
  PGRST_DB_SCHEMAS: public
  PGRST_DB_ANON_ROLE: anon
  PGRST_JWT_SECRET: ${PGRST_JWT_SECRET}    # v0 也設好，預備將來開 auth
  PGRST_SERVER_PORT: 3000
depends_on:
  postgres:
    condition: service_healthy
networks: [internal]
```

`reverse-proxy` route：`/api/*` → `postgrest:3000/*`

## Schema Migration Strategy

統一用 SQL files + 版本化 timestamp 命名，**不**使用 ORM migration 工具：

- 目錄：`infra/migrations/`（待建）
- 命名：`YYYYMMDDHHMMSS_description.sql`（呼應 `.claude/rules/general.md`）
- 套用：建議 `dbmate` 或 `sqitch`（純 SQL 取向），避免 ORM lock-in
- 退路：若選 `dbmate`，遷移狀態存在 `schema_migrations` table

**不**用：
- Supabase CLI migration（耦合 Supabase 工作流）
- Prisma / Drizzle migration（與 Rust 後端不對齊）
- `refinery`（Rust 端 v0 不直接接 postgres，refinery 留給 SQLite 本機 migration）

> 注意：Legacy 的 188 個 migration 不能直接 apply 到新 DB——它們依賴 `auth.users`、Supabase storage 等。Phase 5 migration 工作會撰寫新的 baseline `0001_initial.sql`，並用 data migration script 從 legacy 拉資料。

## Evidence-First Schema Convention

呼應 `data-model-v1.md`：

- 每個 evidence-backed 欄位用單一 `JSONB` column 儲存 `EvidenceValue<T>` 的 serde 表示
- `EvidenceStatus` 由 JSONB `status` field 維護
- Generated column 從 `confirmed` 部分抽出索引欄位（例：`address_city`）
- 整表 RLS：v0 預設 deny；service_role 可寫；anon 可讀（後續開 auth 後換成 per-user）

DDL pattern 見 `data-model-v1.md` §9。

## Consequences

### 正面

- Phase 2 desktop app 可在「server-side Rust 還沒寫」之前先打 PostgREST
- Schema 移植路徑與 legacy 對齊，audit 中標 direct map 的 50+ table 可省工
- 維運只兩個容器，升級壓力小

### 負面 / 成本

- PostgREST 對「跨表 evidence 聚合」query 不友善，未來會逐步把這類 query 移到 Rust axum
- v0 不啟用 RLS for auth 簡化，但要小心 PostgREST `anon` role 對寫入操作的權限收斂（建議 `anon` 只給 SELECT，寫入透過 Rust server-side 走 service_role）
- PostgREST 版本升級偶有 breaking（特別 v11 → v12 的 OpenAPI schema），需追 changelog

## Alternatives Considered

| 方案 | 拒絕理由 |
|:--|:--|
| Supabase Cloud | 違反 PRD §6.2 內網要求 |
| Supabase Self-Hosted 完整 stack | 8+ container，含 realtime / edge / studio / kong，v0 用不到 |
| Plain Postgres + Rust axum API | Day-one 寫 typed API 對 Solo lead 過重；保留為 Phase 4-5 退路 |
| Plain Postgres + NestJS / FastAPI | 與 Rust 後端生態不對齊；增加語言數 |
| SQLite-only（無 server DB） | Cross-device / multi-user 路徑斷死；canonical 在本機違反 PRD §6 |
| CockroachDB / YugaByte 等分散式 | Over-engineering；單機 server 不需分散式 |

## Open Questions

1. **Storage 容器**：Supabase Storage 容器、MinIO、或 `nginx + fs`？建議 MinIO（S3 相容、可獨立升級），但 `nginx + fs` 對單機 server 更簡單。由 `internal-server-plan.md` 決議。
2. **Auth 啟用時機**：v0 不開。多 user / 多裝置或 server expose 給其他電腦時開 GoTrue。屆時新增 ADR-007。
3. **PostGIS 採用時機**：legacy install 了沒用；新版 GIS 進入 scope（Phase 3）時起 ADR-008 決定切換。
4. **RLS 起始策略**：v0 用 service_role bypass + anon read-only。但 PostgREST 直接暴露給 desktop 是否要把寫入全收到 Rust 那層？建議：是。寫入一律 Rust → service_role → postgres。
5. **Connection pooling**：是否上 PgBouncer？v0 單 desktop 不必，後續多 user 再加。
6. **備份工具**：`pg_dump` cron + restic（呼應 `internal-server-plan.md` §6）；確認 server 上裝 restic。

## Migration Path to Plain Postgres + Rust API (退路)

若 Phase 4-5 決定切走 PostgREST：

1. Postgres schema 不動
2. 把 `/api/*` 路由從 PostgREST 改指向 Rust axum service
3. axum 用 `sqlx` 直接 query postgres
4. 桌面 app 的 HTTP client 不必改（URL 不變）
5. RLS policy 改在 axum service 層做（或保留 postgres RLS 加雙保險）

切換成本：寫 axum CRUD ~ 1-2 週工時（依 endpoint 數），但 schema、frontend、桌面 client 都不必動。

## Verification

部署後驗證：

```bash
# Server 端
docker compose ps                                # postgres + postgrest 都 healthy
psql -h 192.168.1.6 -U realestate -c "SELECT version();"
curl http://192.168.1.6:8080/api/                 # PostgREST root 回 schema

# Desktop 端
cargo test --test server_connection              # 整合測試打 PostgREST
```

Review checklist：
- [ ] `postgres` container 不對 host 暴露 port
- [ ] `postgrest` 走 reverse-proxy `/api/*`
- [ ] `.env.local` 有 `PG_PASSWORD` / `PGRST_JWT_SECRET`，不入 git
- [ ] `anon` role 只給 SELECT；寫入由 service_role（Rust server 持有）
- [ ] `pg_data` volume 在 backup 名單
- [ ] PostGIS extension install（但欄位用法見 ADR-008 待擬）
