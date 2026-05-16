# Internal Server Plan

Status: Draft (skeleton + TBD)
Date: 2026-05-17
Owner: Project lead
Related: `docs/product/prd.md` §6.2 §13, ADR-002, ADR-006

## 目的

定義「家中內網 server」上要跑哪些服務、port、volume、env、health check、備份策略。本文件是 PRD §6.2 / §13 的具體化，也是 desktop app config 中 `server.base_url` 指向的服務組合的權威來源。

> CLAUDE.md 規定：server 服務異動必須同步更新本檔。

---

## 1. Server Inventory

| 項目 | 值 |
|:--|:--|
| Hostname / IP | `rick@192.168.1.6` |
| OS | **TBD**（建議 Ubuntu LTS / Debian） |
| CPU / RAM / Disk | **TBD**（建議下限：8C / 32GB / 1TB SSD） |
| Docker / Compose | **TBD**（是否已安裝，版本？） |
| GPU（AI inference 用） | **TBD**（若有，記 model + driver；若無，僅做 routing 不在本機推論） |
| 內網範圍 | `192.168.1.0/24`（待確認） |
| 對外暴露 | **不開**（所有服務僅監聽內網） |
| 訪問方式 | SSH（user `rick`），鍵對驗證 |

**Action**：第一次連上 server 後填齊本表，並把 SSH 公鑰、`sudo` 設定記錄到 `docs/deployment/server-access.md`（待建）。

---

## 2. Network Plan

- **單一 reverse proxy 接住所有 HTTP 流量**（建議 Traefik 或 Caddy）
- 對外（內網其他機器）只暴露 proxy 的單一 port（建議 `:8080`）
- Proxy 後內部服務用 Docker network 互連，**不對 host 開 port**
- mDNS 或 hosts 別名：`realestate.local`（待確認 NAS / router 是否支援）

```text
desktop app  ───► http://192.168.1.6:8080  ───►  reverse-proxy
                                                   ├── /api/      → backend api
                                                   ├── /storage/  → object storage
                                                   ├── /ai/       → ai broker
                                                   └── /health/   → aggregate health
```

---

## 3. Service Inventory

ADR-006 已決定 MVP 採用 Supabase self-hosted lean stack：`postgres:17` + `postgrest:v12`。完整 Supabase stack 的 auth、realtime、edge functions、studio 暫不部署。

### 3.1 必備服務（MVP）

| 服務 | 角色 | 容器映像 |
|:--|:--|:--|
| `reverse-proxy` | 統一入口、內網 route、aggregate health | `caddy:2`（預設）或 `traefik:v3` |
| `postgres` | Canonical 資料庫；安裝 `uuid-ossp`、`pgcrypto`、`postgis` extension | `postgres:17` |
| `postgrest` | `/api/*` 自動 REST API；v0 anon read-only，寫入保留給 Rust service_role path | `postgrest/postgrest:v12` |

### 3.2 MVP 延後服務（Phase 2+）

| 服務 | 角色 | 進場條件 |
|:--|:--|:--|
| `object-storage` | 原始文件 / AI 輸出 blob | Document intake 需要保存原始檔時 |
| `ocr-worker` | PDF/image OCR | Document processing pipeline 進入 Phase 2/3 時 |
| `ai-broker` | LLM provider routing + retry + cost log | AI review pipeline 需要 server-side routing 時 |

### 3.3 第二批（Should Have）

| 服務 | 角色 |
|:--|:--|
| `search` | Full-text / vector search（Postgres FTS / pgvector 或獨立 Meilisearch） |
| `gis-worker` | GIS 圖資查詢與生成 |
| `log-aggregator` | Loki + Promtail，或先用 Docker logging driver |
| `metrics` | Prometheus + Grafana（僅內網） |

### 3.4 不在本 server（明確排除）

- 公網 reverse proxy / DNS（本專案不對外）
- 第三方 AI provider（OpenAI、Anthropic 等）— 直接 outbound call，不本機 host
- Email / SMTP（暫無需求）
- Supabase Studio / Realtime / Edge Functions / GoTrue（v0 不部署）

---

## 4. Per-Service Spec Template

每個服務在實際部署前必須完成下表：

| 欄位 | 範例 |
|:--|:--|
| Service name | `ocr-worker` |
| Container image | `paddleocr/paddleocr:2.7` |
| Internal port | `9000` |
| Exposed via proxy | `/ocr/` |
| Env vars | `OCR_LANGS=ch_tra,en`, `WORKER_THREADS=2` |
| Secret env | （由 server-side secret store 注入，不寫進 compose） |
| Volume | `ocr_models:/models`, `ocr_cache:/cache` |
| Health check | `GET :9000/healthz` 200 |
| Restart policy | `unless-stopped` |
| Backup target | `ocr_models` 不需備份；`ocr_cache` 可清 |
| Logs | stdout → Docker driver |
| Notes | GPU optional |

各服務檔案存於 `docs/deployment/services/<service>.md`。

---

## 5. Compose / Orchestration

第一版用 `docker compose`，分檔組合：

```
infra/
├── compose.proxy.yml      reverse-proxy
├── compose.data.yml       postgres + postgrest
├── compose.storage.yml    object-storage（Phase 2+）
├── compose.ai.yml         ocr-worker + ai-broker（Phase 3+）
└── compose.observability.yml  log + metrics（後加）
```

啟動：

```bash
docker compose \
  -f compose.proxy.yml \
  -f compose.data.yml \
  up -d
```

未來若服務數 > 8 或需要 rolling update，再評估 Kubernetes / k3s（**目前不上**，避免 over-engineering）。

---

## 6. Volume & Backup

### 6.1 Volume 規劃

| Volume | 內容 | 備份頻率 | 備份方式 |
|:--|:--|:--|:--|
| `pg_data` | Postgres canonical 資料 | **每日** | `pg_dump` → 加密壓縮 → 另一顆硬碟 |
| `object_data` | 原始文件、AI 輸出 blob（Phase 2+） | **每日** | rsync / restic → 另一顆硬碟 |
| `ai_logs` | AI request/response logs（Phase 3+） | 每週 | tar + 移到 cold storage |
| `proxy_certs` | TLS cert | 每次變更 | git commit（加密） |
| `ocr_cache` / `model_cache` | 快取 | 不備份 | 重抓即可 |

### 6.2 備份保留策略

- 每日備份保留 14 天
- 每週備份保留 8 週
- 每月備份保留 12 個月
- **異地副本**：建議另一台機器或外接硬碟（**TBD**：是否有 NAS / 雲端冷儲存）

### 6.3 Restore Drill

每 90 天執行一次「假裝壞掉」演練：
1. 在獨立目錄還原備份
2. 啟一份 Postgres + object-storage
3. 驗證最近 7 日 property / document / run 是否能讀
4. 結果寫入 `docs/deployment/restore-drill-log.md`

---

## 7. Health Check 統一規格

每個服務必須提供：

- `GET /healthz` → 200 OK（liveness）
- `GET /readyz` → 200 OK / 503（readiness，含依賴檢查）

`reverse-proxy` 提供 aggregate：

```
GET http://192.168.1.6:8080/health
{
  "overall": "degraded",
  "checked_at": "2026-05-17T...",
  "services": [
    { "name": "postgres", "status": "ok", "latency_ms": 3 },
    { "name": "postgrest", "status": "ok", "latency_ms": 6 },
    { "name": "object-storage", "status": "not_configured" },
    { "name": "ai-broker", "status": "not_configured" }
  ]
}
```

Desktop app 用這個 endpoint 顯示「degraded mode」UI（PRD §16.5）。

---

## 8. Security

- 所有服務只 bind 內網 IP，不 bind `0.0.0.0`
- Postgres 不對 host 暴露 port；PostgREST 只透過 reverse-proxy `/api/*` 暴露
- object-storage 上線前需新增獨立 service spec，不與 Postgres volume 混放
- Secret（DB password, AI API keys）用 `.env.local` 注入 compose，**不進 git**
- `.env.local` 用 `age` 或 GPG 加密備份
- API → DB 走 service network，不經 proxy
- TLS：內網用自簽 + 提示信任根；對外不開
- 內網非授權設備不能掃到 port → 確認 router 不 broadcast

---

## 9. Logging & Monitoring（Phase 2+）

第一版：

- 所有 container 用 Docker default logging
- 必要時 `docker compose logs -f <service>` 看
- 每週看一次磁碟用量

第二版（Should Have）：

- Loki + Promtail 集中 log
- Prometheus + Grafana 看 CPU / mem / DB conn 數
- AI broker 額外記 token / cost / provider success rate

---

## 10. Deployment Lifecycle

### 第一次部署

1. SSH 進 server，安裝 Docker / Compose
2. clone deployment 設定（**TBD**：要不要把 compose 檔放本 repo `infra/` 目錄，或另起一個 deployment repo）
3. 準備 `.env.local`（手動填 secret）
4. `docker compose ... up -d`
5. 跑 healthcheck
6. desktop app 設定 `server.base_url`，連線驗證

### 服務升級

- Patch（同 minor）：直接 pull + restart
- Minor / Major：先在獨立 compose project 跑、驗 health、切流量

### Rollback

- Volume 永遠保留前一次備份
- Compose tag pin 在固定 major.minor，避免亂升

---

## 11. Open Questions

1. **server OS、Docker 狀態未知** — 第一次連上後確認並填表
2. **GPU 是否存在** — 影響 OCR / AI inference 部署
3. **異地備份目的地** — NAS / 外接硬碟 / 雲端冷儲存 三選一
4. **內網是否需要 mDNS / DNS 別名**
5. **deployment repo 與 app repo 是否分開** — 建議分開（infra 變動跟 app code 解耦），但第一版可放本 repo `infra/` 暫存
6. **rick 帳號權限** — 是否能直接 `sudo` 無密碼？SSH 用 key 還是密碼？
7. **object storage 選型** — MinIO 或 fs + nginx，等 Document intake 進入 Phase 2 再決定

---

## 12. Verification

部署完成後在 desktop app 預期可見：

- 服務狀態頁顯示所有服務 ✓
- 拔掉 `ai-broker` 後，UI 顯示 degraded 而非整個 app 掛掉
- 重啟 server 後 desktop app 自動重連，pending queue 自動推送

驗收 checklist 各服務獨立寫在 `docs/deployment/services/<service>.md`。
