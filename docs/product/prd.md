# Realestate_Management_Apps PRD

Status: Draft v0.1  
Date: 2026-05-17  
Owner: Project lead  
Repo: `jason660519/Realestate_Management_Apps`

## 1. Product Summary

Realestate_Management_Apps 是舊 `Owner-Property-Management-AI-SPA` 的新一代桌面版重建專案。舊方案不動，新 repo 以 Rust + Tauri 建立 desktop app，並把重型 Docker 服務移到家中內網 server `rick@192.168.1.6`。

本產品不是單純把舊 Next.js SPA 包成桌面殼，而是重建成一個「房地產營運工作台」：物件管理、謄本/文件判讀、GIS 圖資、AI review、任務協作、跨 app plugin 都要在同一個可控、可追蹤、可離線降級的桌面環境中運作。

## 2. Strategic Goals

1. 建立新的 Rust + Tauri desktop app，不破壞舊 SPA。
2. 把 Docker-based backend/runtime 服務集中到內網 server，桌面 app 只保留必要 local state。
3. 讓 Realestate_Management_Apps、SayDo、Project-Manager 未來能透過明確 plugin contract 互相接入。
4. 延續舊系統已驗證的房產 domain workflow，但重做資訊架構與操作體驗。
5. 高風險 AI 流程必須 evidence-first，不產生假資料，不默默 fallback。

## 3. Non-Goals

1. 不在本 repo 直接修改舊 `Owner-Property-Management-AI-SPA`。
2. 第一階段不追求完整複製舊 SPA 所有 superadmin 功能。
3. 不讓三個 app 共用未文件化的資料庫、secret、local state 或 Docker volume。
4. 不把 server IP 硬編進 source module。
5. 不做公開 marketing website；第一畫面就是可工作的營運 app。

## 4. Target Users

## 4.1 Primary User: 房產營運管理者

需要管理物件、屋主/客戶資料、文件、謄本、GIS 圖資、租售流程與 AI 生成內容。重點是準確、可追溯、可快速修正。

## 4.2 Secondary User: AI/Operations Engineer

需要設定模型、檢查 AI pipeline、處理失敗任務、監控內網服務、管理 plugin 權限與 deployment 狀態。

## 4.3 Supporting User: 協作/任務管理者

透過 Project-Manager 接收房產 app 的待辦、錯誤、驗收、資料補件任務。

## 5. Product Principles

1. Evidence first: 高風險資料必須顯示來源、信心、差異與確認狀態。
2. Human before canonical save: 謄本、權狀、GIS、法律/坪數資料需先確認再寫入正式物件資料。
3. Local desktop, service-backed: UI 和安全邊界在 desktop，重型服務在內網 server。
4. Explicit plugin contracts: 三 app 互通靠 contract，不靠偷讀資料。
5. Visible degraded mode: server、模型、API key、plugin 失敗時要明確顯示。
6. No fabricated data: AI 不能判斷時要失敗並說明原因，不可補假資料。

## 6. Platform Architecture Direction

## 6.1 Desktop App

- Runtime: Rust + Tauri.
- UI: web frontend inside Tauri WebView.
- Privileged operations: Rust command surface.
- Local storage: app-scoped settings, cache, queue metadata, recent workspace state.
- Security: WebView 不直接擁有任意 filesystem、shell、secret 權限。

## 6.2 Internal Server

Target:

```text
rick@192.168.1.6
```

Server responsibilities:

- Database/runtime services.
- AI worker services.
- OCR/PDF/GIS processing containers.
- Search/indexing services if needed.
- Health checks and logs.
- Backup-ready data volumes.

Each service must have separate env, data volume, port, health check, and backup note under `docs/deployment/`.

## 6.3 Legacy System Position

舊 `Owner-Property-Management-AI-SPA` 是 reference system：

- 可參考資料表設計。
- 可參考已驗證 workflow。
- 可參考 prompt/model routing 策略。
- 可作為 migration source。

但舊系統不作為新 desktop app 的 runtime dependency。

## 7. Multi-App Plugin Strategy

## 7.1 SayDo Integration

Potential capabilities:

- 語音輸入到房產工作台欄位。
- 口述轉物件摘要、帶看紀錄、客戶需求。
- prompt-scoped model routing 作為可參考能力。

Boundary:

- Realestate_Management_Apps 不讀 SayDo local database。
- SayDo 透過明確 plugin/export/API contract 提供文字或 task result。
- 語音原始檔與 transcript 權限需明確顯示。

## 7.2 Project-Manager Integration

Potential capabilities:

- 房產 app 產生任務：文件缺件、AI 判讀失敗、資料待確認、GIS 待人工補件。
- Project-Manager 顯示狀態、負責人、驗收、blocking issues。
- Project-Manager 回寫任務完成/退回結果。

Boundary:

- 任務 contract 需 versioned schema。
- 不直接共用內部資料表。
- 任務狀態同步失敗時，房產 app 保留 local pending state。

## 7.3 Realestate_Management_Apps as Provider

Potential outbound capabilities:

- Property summary.
- Document evidence packet.
- AI review result.
- GIS asset status.
- Taskable issue export.

## 8. MVP Scope

MVP 目標是建立可運作的 desktop shell、server connection、核心資料模型和一條 evidence-first 文件工作流。

## 8.1 MVP Must Have

1. Desktop app shell
   - App navigation.
   - Settings.
   - Service health status.
   - Plugin status panel.

2. Internal server connection
   - Server base URL configurable.
   - Health check visible.
   - Offline/degraded state visible.

3. Property workbench
   - 物件列表。
   - 物件基本資料。
   - 文件/圖資區。
   - AI review/status 區。

4. Document intake
   - Upload/import local PDF/image/doc files.
   - Show file type, processing route, and status.
   - Keep original files separate from AI outputs.

5. Transcript/GIS evidence workflow foundation
   - 不要求第一版完成所有舊系統 parser。
   - 第一版要建立 stage model: detect -> parse -> review -> human confirm -> save.
   - 每個 stage 要有 status、provider/model、error、source reference。

6. Plugin registry foundation
   - List known peers: SayDo, Project-Manager.
   - Show enabled/disabled/permission status.
   - No hidden cross-app state sharing.

## 8.2 MVP Should Have

1. Queue and retry view for failed AI/server jobs.
2. Manual upload fallback for GIS or official-source documents.
3. Basic task export to Project-Manager contract draft.
4. Local app diagnostics export.

## 8.3 MVP Not Yet

1. Full mobile app replacement.
2. Full old superadmin parity.
3. Public web portal.
4. Automated production migration.
5. Multi-user permission system beyond local operator/admin mode.

## 9. Core User Journeys

## 9.1 新增物件

1. User creates property.
2. App saves local/server draft.
3. User uploads documents and photos.
4. App shows processing state.
5. AI output appears as review draft.
6. User confirms or edits.
7. Canonical property data updates only after confirmation.

## 9.2 文件判讀

1. User imports PDF/image.
2. App classifies file and chooses route.
3. Server-side worker processes document.
4. App displays stage trace and evidence.
5. If confidence is low or model fails, app shows failure with next action.
6. User confirms extracted values before save.

## 9.3 GIS/圖資流程

1. Property has address/land number.
2. App requests GIS-related generation or lookup.
3. Generated or uploaded files are stored as document assets.
4. App displays source, generated time, and status.
5. Manual fallback remains available.

## 9.4 跨 app 任務

1. AI review finds missing data.
2. User creates task.
3. Project-Manager receives task payload.
4. Project-Manager returns status.
5. Realestate app reflects task state without owning Project-Manager internals.

## 10. Data Domains

Initial domains:

- Properties.
- People and organizations.
- Documents.
- Document processing runs.
- AI stage traces.
- GIS assets.
- Tasks and plugin handoffs.
- Settings and service connections.
- Audit/events.

Data that affects legal/property truth must distinguish:

- Source value.
- AI extracted value.
- Human edited value.
- Confirmed canonical value.
- Timestamp and actor.

## 11. AI Requirements

## 11.1 Model Routing

AI modules should support:

- Module-specific provider/model selection.
- Fallback chain.
- Visible provider/model failure.
- Per-stage logs and cost/token estimates where available.

## 11.2 High-Risk Rules

For transcript, GIS, ownership, area, legal, financial, or contract data:

- No fabricated values.
- No silent fallback to seeded data.
- Show source evidence.
- Require human confirmation before canonical save.
- Keep raw original document intact.

## 11.3 Workbench Trace

Every AI run should expose:

- Stage name.
- Input document/page reference.
- Provider/model.
- Status.
- Extracted output.
- Confidence or disagreement state.
- Error message.
- Retry/manual fallback path.

## 12. UX Requirements

## 12.1 Main Navigation

Initial navigation:

- Workbench.
- Properties.
- Documents.
- AI Review.
- Tasks.
- Integrations.
- Settings.

## 12.2 App Shell

- Dense operational layout.
- Persistent status area for server/plugin health.
- Split panel for list/detail/review workflows.
- No marketing hero screen.
- Visible empty/loading/error/blocked states.

## 12.3 Review UX

AI output must be editable before save. For evidence-backed extraction, show source beside extracted value where practical.

## 13. Deployment Requirements

1. Server address is environment/config, not source logic.
2. Each Docker service has separated volume and env.
3. Every service has health check.
4. Desktop app shows connection state.
5. Backup/restore process must be documented before production data migration.

## 14. Security and Privacy Requirements

1. Secrets are never shown raw.
2. Plugin permissions are explicit.
3. Tauri privileged operations stay in Rust command layer.
4. File access is scoped to user-selected files/directories and app data.
5. AI provider credentials are app-scoped or server-scoped by documented decision.
6. Cross-app access requires permission and versioned contract.

## 15. Migration Strategy

## 15.1 Phase 0: Product and Architecture

- PRD.
- Architecture ADRs.
- Deployment plan.
- Plugin contract drafts.

## 15.2 Phase 1: Desktop Foundation

- Scaffold Tauri app.
- App shell.
- Settings.
- Health checks.
- Plugin registry placeholder.

## 15.3 Phase 2: Property and Document MVP

- Property CRUD.
- Document intake.
- Basic processing run model.
- Evidence review draft.

## 15.4 Phase 3: AI/GIS Workflows

- Transcript processing stages.
- GIS asset workflow.
- Manual fallback.
- AI monitor/retry surfaces.

## 15.5 Phase 4: Cross-App Integration

- SayDo input contract.
- Project-Manager task contract.
- Realestate provider contract.

## 15.6 Phase 5: Legacy Migration

- Read-only migration tooling.
- Sample migration.
- Verification report.
- Production migration approval.

## 16. Success Metrics

1. User can launch desktop app and see server/plugin health.
2. User can create a property and attach documents.
3. User can run a document processing workflow and inspect stage trace.
4. High-risk AI output cannot become canonical without confirmation.
5. App can show degraded mode when server is unavailable.
6. Project-Manager and SayDo integration points are documented before implementation.
7. No direct hidden state sharing exists between apps.

## 17. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Rebuilding too much old SPA at once | Slow delivery | MVP focuses on shell, property, document, evidence workflow |
| Server becomes single point of failure | Desktop app blocked | Health checks, degraded mode, local queue metadata |
| AI creates incorrect property/legal data | High trust risk | Evidence-first review and human confirmation |
| Plugin boundaries become implicit | Cross-app contamination | Versioned contracts and permission scope |
| Docker volumes/secrets mix across apps | Operational contamination | Per-project env, volume, service docs |
| Migration loses provenance | Legal/data risk | Preserve source, extracted, edited, confirmed fields |

## 18. Open Questions

1. 新 app 第一個正式資料庫要沿用 Supabase/Postgres，還是先用 server-hosted Postgres + typed API？
2. Desktop app 是否需要多使用者登入，或第一版只做 local operator/admin？
3. 舊 SPA 哪些資料表是 Phase 2 必須遷移，哪些只保留 reference？
4. GIS 服務要自建 worker，還是沿用官方網站/manual fallback 為主？
5. SayDo 與 Project-Manager 第一個 plugin contract 要先做哪一條？

## 19. First Implementation Recommendation

先做 Phase 0 + Phase 1：

1. 補 `ADR-002-tauri-service-architecture.md`。
2. 補 `ADR-003-plugin-contract-boundary.md`。
3. 補 `docs/deployment/internal-server-plan.md`。
4. Scaffold Rust + Tauri app。
5. 建立 app shell、settings、server health、plugin registry placeholder。

這樣可以先把桌面架構、內網服務邊界、三 app plugin 邊界立住，再開始搬房產 domain 功能。
