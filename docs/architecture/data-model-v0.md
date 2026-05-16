# Data Model v0 (Draft)

Status: **Superseded by `data-model-v1.md`**（2026-05-17）
Date: 2026-05-17
Scope: Phase 0 設計輸入，作為 ADR-002 與 scaffold 準備依據
Related: `docs/product/prd.md` §10 §11, ADR-002

> **本檔已被 v1 取代**。保留作為歷史快照；新工作請看 `data-model-v1.md`。

## Scope

本草案只定義四個核心 entity，作為 scaffold 與 Phase 2 起點：

- `EvidenceValue<T>` — 橫切型，evidence-first 的具體結構
- `Property` — 物件
- `Document` — 文件
- `ProcessingRun` / `AIStageTrace` — AI 處理 trace

其他 PRD §10 列出的 entity（People、GIS、Tasks、Audit、Settings）待後續迭代擴充。

## Design Principles

1. **Evidence-first**：任何可能被 AI 影響、或會影響法律 / 物權判斷的欄位，都包進 `EvidenceValue<T>`
2. **Confirmed = canonical**：沒有 `confirmed` 不算正式資料；AI **不能**寫 `confirmed`
3. **Append-only audit**：第一版用「最後寫入勝出 + confirmed 鎖定」+ 獨立 `audit_event` log，不在 entity 內版本化欄位
4. **Soft references**：document 與 property 用 UUID 連結，避免 strict FK 限制 draft 流程
5. **Locale-aware**：地址、坪數、ownership 預留繁體中文欄位與台灣量度單位（坪 / 平方公尺）

## Core Type: EvidenceValue<T>

Evidence-first 的核心容器。所有可能被 AI 抽取或人工修正的欄位都用這個 wrapper。

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceValue<T> {
    pub source: Option<EvidenceSource>,

    pub ai_extracted: Option<T>,
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub ai_confidence: Option<f32>,
    pub ai_extracted_at: Option<DateTime<Utc>>,
    pub ai_stage_id: Option<Uuid>,  // 反查 AIStageTrace

    pub human_edited: Option<T>,
    pub human_edited_by: Option<ActorId>,
    pub human_edited_at: Option<DateTime<Utc>>,

    pub confirmed: Option<T>,
    pub confirmed_by: Option<ActorId>,
    pub confirmed_at: Option<DateTime<Utc>>,

    pub status: EvidenceStatus,
    pub failure_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Empty,        // 從未動過
    Pending,      // AI 處理中
    AiDone,       // 有 ai_extracted, 待人工 review
    Edited,       // 人工改過, 待 confirm
    Confirmed,    // canonical
    Failed,       // AI 失敗
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSource {
    pub document_id: Uuid,
    pub page: Option<u32>,
    pub bbox: Option<BBox>,            // normalized 0-1
    pub raw_text: Option<String>,
    pub external_uri: Option<String>,  // 官方來源 URL (e.g. GIS 官網)
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
```

### Helper API

```rust
impl<T: Clone> EvidenceValue<T> {
    /// 唯一可作為 canonical 的 getter。沒 confirmed 一律 None。
    pub fn canonical(&self) -> Option<&T> {
        self.confirmed.as_ref()
    }

    /// 目前 UI 應顯示的「待確認值」：human_edited 優先於 ai_extracted。
    pub fn current_proposal(&self) -> Option<&T> {
        self.human_edited.as_ref().or(self.ai_extracted.as_ref())
    }

    pub fn is_canonical(&self) -> bool {
        matches!(self.status, EvidenceStatus::Confirmed) && self.confirmed.is_some()
    }
}
```

### Invariants（必須由 service 層強制）

1. AI 寫入路徑**不可**碰 `confirmed_*` 欄位
2. `confirmed` 為 `Some` 時 `status` 必須是 `Confirmed`
3. 已 `Confirmed` 後若要再變更，必須先進入 `Edited`，再由人工重新 confirm（不可 in-place 覆寫）
4. `Failed` 狀態必須有 `failure_reason`
5. AI 重抽（同一 evidence 再跑一次）只能寫 `ai_extracted` 並把 status 退回 `AiDone`；不會清掉已存在的 `confirmed`

建議寫成 invariant test 守住 1, 2, 3。

## Property

```rust
pub struct Property {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_actor: ActorId,            // 建立者（本機操作者）
    pub draft_state: DraftState,         // LocalOnly | Synced | Conflict

    // 高風險欄位（evidence-backed）
    pub address: EvidenceValue<Address>,
    pub land_number: EvidenceValue<String>,         // 地號
    pub building_no: EvidenceValue<String>,         // 建號
    pub building_area: EvidenceValue<AreaMeasure>,  // 坪數
    pub ownership: EvidenceValue<OwnershipInfo>,

    // 低風險欄位（使用者直接輸入，免 evidence）
    pub display_name: String,
    pub property_type: PropertyType,     // Residential | Commercial | Land | ...
    pub status: PropertyStatus,          // ForSale | ForRent | Sold | Archived
    pub tags: Vec<String>,
    pub notes: String,
}

pub struct Address {
    pub raw: String,                     // 完整地址原始字串
    pub city: Option<String>,
    pub district: Option<String>,
    pub road_or_section: Option<String>,
    pub lane: Option<String>,
    pub alley: Option<String>,
    pub number: Option<String>,          // 「123 之 1 號」整段
    pub floor: Option<String>,
    pub unit: Option<String>,
    pub postal_code: Option<String>,
}

pub struct AreaMeasure {
    pub ping: Option<f64>,               // 坪
    pub sqm: Option<f64>,                // 平方公尺
    pub measure_basis: AreaBasis,        // Main | Accessory | Public | IncludesParking
}

pub struct OwnershipInfo {
    pub owners: Vec<OwnerShare>,         // 多人共有
    pub registered_at: Option<NaiveDate>,
}

pub struct OwnerShare {
    pub name: String,
    pub share_numerator: u32,
    pub share_denominator: u32,
    pub note: Option<String>,
}
```

設計筆記：
- `address` 整包包 `EvidenceValue<Address>`，因為謄本上地址是連續文本，AI 抽取以整段為單位
- 結構化 query 用 server 端 derive 出的 `city` / `district` index column，不影響 entity 設計
- `ownership` 同上：多人共有的份額是整體事實，整包 evidence 較合理

## Document

```rust
pub struct Document {
    pub id: Uuid,
    pub property_id: Option<Uuid>,              // 可暫不關聯
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: ActorId,
    pub kind: EvidenceValue<DocumentKind>,      // AI 分類也走 evidence
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub storage: DocumentStorage,
    pub current_run: Option<Uuid>,
    pub run_history: Vec<Uuid>,
}

pub struct DocumentStorage {
    pub original_uri: String,                   // server-side canonical path
    pub local_cache_path: Option<String>,       // Tauri local cache
    pub checksum_sha256: String,
}

pub enum DocumentKind {
    LandRegistry,        // 謄本
    OwnershipCert,       // 權狀
    Contract,            // 合約
    Floorplan,           // 平面圖
    GisAsset,            // GIS 圖資
    Photo,
    Other,
}
```

## ProcessingRun / AIStageTrace

```rust
pub struct ProcessingRun {
    pub id: Uuid,
    pub document_id: Uuid,
    pub property_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub overall_status: RunStatus,
    pub stages: Vec<AIStageTrace>,
}

pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct AIStageTrace {
    pub stage_id: Uuid,
    pub stage_name: StageName,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    pub provider: Option<String>,
    pub model: Option<String>,
    pub status: StageStatus,

    pub input_ref: StageInputRef,
    pub output: Option<serde_json::Value>,  // schema 依 stage_name 不同
    pub confidence: Option<f32>,
    pub disagreements: Option<Vec<Disagreement>>,
    pub error: Option<StageError>,

    pub tokens_used: Option<u32>,
    pub cost_estimate_usd: Option<f32>,
    pub retry_of: Option<Uuid>,             // 前一次失敗的 stage_id
}

pub enum StageName {
    Detect,        // 判斷 DocumentKind
    Parse,         // OCR / structure extraction
    Extract,       // 抽 evidence value (Address, Ownership, ...)
    Review,        // 跨欄位 sanity check
    HumanConfirm,  // 使用者動作（不會由 AI 觸發）
    Save,          // 寫入 canonical（confirmed 之後）
}

pub enum StageStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

pub struct StageInputRef {
    pub document_id: Uuid,
    pub page_range: Option<(u32, u32)>,
    pub previous_stage: Option<Uuid>,
}

pub struct Disagreement {
    pub field: String,
    pub provider_a: String,
    pub value_a: serde_json::Value,
    pub provider_b: String,
    pub value_b: serde_json::Value,
}

pub struct StageError {
    pub kind: StageErrorKind,
    pub message: String,
    pub retryable: bool,
}

pub enum StageErrorKind {
    ProviderTimeout,
    ProviderRateLimit,
    InvalidOutput,
    LowConfidence,
    HumanRejected,
    Unknown,
}
```

## Storage Mapping (informative)

實際 schema migration 等 ADR-004（storage 決策）後再定型。以下為設計時的預期方向：

**Server 端（Postgres / Supabase 待定）：**
- `properties` table — evidence 欄位以 `JSONB` 內嵌（每欄位一個 JSONB column）
- `documents` table
- `processing_runs` table — `stages` 用 `JSONB[]` 或 normalized 子表（視 query 模式定）
- `audit_events` table — evidence 變更歷史獨立寫入

**Local 端（Tauri SQLite）：**
- `property_summary` — list view cache
- `property_drafts` — 未同步的 draft
- `pending_imports` — server unreachable 時的匯入 queue

## Open Questions

1. `Address` 子欄位是否獨立 `EvidenceValue` 還是整包？
   **建議**：整包。理由：謄本 / 合約上地址是連續文本，AI 抽取以整段為單位。結構化查詢用 server-derived index column。

2. Evidence 修改是否要 entity 內版本化（embedded history）？
   **建議**：第一版**不在 entity 內**版本化。改用獨立 `audit_event` log 記錄每次 evidence 變更，entity 只存「現在的狀態」。理由：避免 entity 表膨脹，audit log 也滿足合規追溯。

3. `Property` 刪除策略？
   **建議**：soft delete（`status = Archived` + `deleted_at` / `deleted_by`），不 hard delete。

4. AIStageTrace 與 EvidenceValue 反查關聯？
   **決策**：已在 `EvidenceValue` 加 `ai_stage_id` 欄位，可從 evidence 反查 stage trace。

5. `OwnerShare.name` 是否需要連結到 People entity？
   **建議**：第一版只存 name（freeform），People entity 進入 Phase 2 後再 normalize。

## Next Steps

1. 等 ADR-004（storage / secret 決策）→ 決定 EvidenceValue 在 Postgres 是 JSONB 內嵌還是獨立表
2. Property 之外的 entity（People、GIS、Tasks）等 Phase 2 開工前再擴
3. 寫 invariant test：confirmed 設定後 AI 不能覆寫；status 與 confirmed 一致性
4. 用此草案逆向檢查舊 SPA schema，標出哪些舊欄位需要 evidence wrapper、哪些可直接搬
