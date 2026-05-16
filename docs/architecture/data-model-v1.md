# Data Model v1

Status: Current（supersedes `data-model-v0.md`）
Date: 2026-05-17
Owner: Project lead
Related: ADR-002, ADR-006, `docs/migration/legacy-schema-audit.md`, `docs/product/prd.md` §10 §11

## Changelog from v0

依 `legacy-schema-audit.md` 對齊：

1. **StageName 對齊 legacy `transcript_intake_runs` 5-stage FSM**
   `Detect / Parse / Review / HumanConfirm / Save`，移除 v0 的獨立 `Extract`（合併進 `Parse`）。
2. **Property 重構為單表 + `kind` enum**
   合併 legacy `property_sales` + `property_rentals`；evidence-relevant 欄位列出清單。
3. **`EvidenceValue<T>` 在 Postgres 用 JSONB column 內嵌**
   給出具體 DDL 與 check constraint。
4. **新增 `AuditEvent`**
   獨立 append-only log，欄位級粒度（legacy 缺）。
5. **PostGIS 暫不採用**
   Flat `latitude` / `longitude` `double precision`，與 legacy 一致；未來 ADR-008 處理切換。
6. **`property_environment_conditions` 整包 wrap**
   80+ OCR 候選欄位用 `EvidenceValue<EnvironmentConditions>` 整包包；v1 不展開欄位。
7. **`property_au_details` defer**
   v1 只台灣，schema 不入 v1，留 ADR 預定（i18n 時補）。

---

## Design Principles

1. **Evidence-first**：所有可能被 AI 影響或影響法律 / 物權判斷的欄位都包 `EvidenceValue<T>`
2. **Confirmed = canonical**：沒 `confirmed` 不算正式資料；AI 不能寫 `confirmed`
3. **Append-only field-level audit**：evidence 變更額外寫入獨立 `audit_events` table
4. **Soft references**：document 與 property 用 UUID 連結，避免 strict FK 限制 draft 流程
5. **Locale-first Taiwan**：v1 只做台灣，i18n 欄位 defer
6. **Server is canonical**：本機 SQLite 只快取 / draft / queue（ADR-004），canonical 在 Postgres

---

## Core Type: `EvidenceValue<T>`

結構與 v0 相同，加上 `ai_stage_id` 反查欄位（已在 v0 加入）：

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
    pub ai_stage_id: Option<Uuid>,

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
    Empty, Pending, AiDone, Edited, Confirmed, Failed,
}

pub struct EvidenceSource {
    pub document_id: Uuid,
    pub page: Option<u32>,
    pub bbox: Option<BBox>,           // normalized 0-1
    pub raw_text: Option<String>,
    pub external_uri: Option<String>,
}
```

### Invariants（service 層強制）

1. AI 寫入路徑**不可**碰 `confirmed_*`
2. `confirmed.is_some()` ⇔ `status == Confirmed`
3. 已 `Confirmed` 後要再變更，先進入 `Edited`（不可 in-place 覆寫）
4. `Failed` 必有 `failure_reason`
5. AI 重抽只能寫 `ai_extracted` 並退回 `AiDone`，不動 `confirmed`
6. **每次 evidence 變更同時寫入 `audit_events`**（新增規則）

---

## Property（v1：合併 sales + rentals）

```rust
pub struct Property {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_actor: ActorId,
    pub draft_state: DraftState,         // LocalOnly | Synced | Conflict
    pub kind: PropertyKind,              // Sale | Rental | LandOnly | Commercial

    // 高風險 evidence-backed
    pub address: EvidenceValue<Address>,
    pub land_number: EvidenceValue<String>,
    pub building_no: EvidenceValue<String>,
    pub building_area: EvidenceValue<AreaMeasure>,
    pub land_area: EvidenceValue<AreaMeasure>,
    pub ownership: EvidenceValue<OwnershipInfo>,
    pub building_age: EvidenceValue<BuildingAge>,
    pub floor_info: EvidenceValue<FloorInfo>,
    pub parking: EvidenceValue<ParkingInfo>,
    pub current_loan: EvidenceValue<LoanInfo>,
    pub environment: EvidenceValue<EnvironmentConditions>, // 80+ 欄整包

    // 低風險 user input
    pub display_name: String,
    pub property_type: PropertyType,
    pub status: PropertyStatus,
    pub tags: Vec<String>,
    pub notes: String,

    // Sale-specific（kind == Sale 時才填）
    pub sale_price: Option<MoneyAmount>,
    pub commission_rate: Option<f32>,

    // Rental-specific（kind == Rental 時才填）
    pub monthly_rent: Option<MoneyAmount>,
    pub deposit_amount: Option<MoneyAmount>,
    pub lease_term_months: Option<u32>,
}

pub enum PropertyKind { Sale, Rental, LandOnly, Commercial }
pub enum PropertyType { Residential, Apartment, Office, Retail, Industrial, Land }
pub enum PropertyStatus { Draft, Active, UnderContract, Sold, Rented, Archived }

pub struct Address {
    pub raw: String,
    pub city: Option<String>,
    pub district: Option<String>,
    pub road_or_section: Option<String>,
    pub lane: Option<String>, pub alley: Option<String>,
    pub number: Option<String>,
    pub floor: Option<String>, pub unit: Option<String>,
    pub postal_code: Option<String>,
}

pub struct AreaMeasure {
    pub ping: Option<f64>, pub sqm: Option<f64>,
    pub measure_basis: AreaBasis,
}
pub enum AreaBasis { MainBuilding, Auxiliary, Common, Parking, Extension, Total }

pub struct OwnershipInfo {
    pub owners: Vec<OwnerShare>,
    pub registered_at: Option<NaiveDate>,
}
pub struct OwnerShare {
    pub name: String,
    pub id_number_hash: Option<String>,  // 不存原值，存 sha256
    pub share_numerator: u32, pub share_denominator: u32,
    pub note: Option<String>,
}

pub struct BuildingAge { pub years: Option<f32>, pub built_at: Option<NaiveDate> }
pub struct FloorInfo { pub current: Option<i32>, pub total: Option<i32>, pub has_basement: bool }
pub struct ParkingInfo { pub count: u32, pub kind: ParkingKind, pub price_included: bool }
pub enum ParkingKind { None, Mechanical, Flat, Mixed }
pub struct LoanInfo { pub bank: Option<String>, pub balance_twd: Option<f64>, pub monthly_payment_twd: Option<f64> }
pub struct MoneyAmount { pub twd: f64, pub recorded_at: NaiveDate }

pub struct EnvironmentConditions {
    /// Legacy property_environment_conditions ~80 欄整包，v1 用 serde_json::Value 暫存
    pub raw: serde_json::Value,
    pub schema_version: u32,
}
```

### Evidence Wrap 分類（依 audit §5.1）

| Legacy 欄位群 | 新版作法 |
|:--|:--|
| `address_*`（city/district/...） | 整包 `EvidenceValue<Address>` |
| `area_main_building/auxiliary/...` × 11 | 收斂為 `building_area` + `land_area`，basis 用 enum |
| `owner_id_number_enc` | hash sha256 存 `id_number_hash`，原值不存 DB；明文留 evidence source PDF |
| `building_age_years` / `built_year` | `EvidenceValue<BuildingAge>` |
| `current_loan_amount` / `loan_bank` | `EvidenceValue<LoanInfo>` |
| `environment_*` × 80 | `EvidenceValue<EnvironmentConditions>` 整包 |
| `is_data_open` / `is_investigation_signed` | 純使用者 toggle，不 wrap |
| `data_source` | 散落，由 evidence.source 自然替代，**drop** |
| `details JSONB`（legacy 後備袋） | **drop**，要求遷移時必須拆 |
| `workflow_control` | **drop**（語義不清，見 audit Open Q §9）|
| `au_*` 系列 | **defer** to i18n ADR |

---

## Document

```rust
pub struct Document {
    pub id: Uuid,
    pub property_id: Option<Uuid>,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: ActorId,
    pub kind: EvidenceValue<DocumentKind>,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub storage: DocumentStorage,
    pub current_run: Option<Uuid>,
    pub run_history: Vec<Uuid>,
    pub version: u32,
    pub replaces_document_id: Option<Uuid>,    // legacy property_documents linked-list
    pub tags: Vec<String>,
    pub is_active: bool,
}

pub enum DocumentKind {
    LandRegistry,        // 謄本
    OwnershipCert,       // 權狀
    Contract,            // 合約
    InvestigationReport, // 不動產說明書
    Floorplan,
    Panorama,
    GisAsset,
    IdentityDocument,
    Photo,
    Other,
}
```

---

## ProcessingRun / AIStageTrace（對齊 legacy 5-stage FSM）

```rust
pub struct ProcessingRun {
    pub id: Uuid,
    pub document_ids: Vec<Uuid>,             // legacy source_document_ids[]
    pub property_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub overall_status: RunStatus,
    pub current_phase: Option<StageName>,
    pub stages: Vec<AIStageTrace>,
    pub error_message: Option<String>,
}

pub enum RunStatus {
    Uploaded, RouteSelected, Detecting, Parsing, Reviewing,
    NeedsUserConfirmation, Confirmed, Failed, Cancelled,
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
    pub output: Option<serde_json::Value>,
    pub confidence: Option<f32>,
    pub consensus_metadata: Option<serde_json::Value>, // 從 legacy property_documents 繼承
    pub disagreements: Option<Vec<Disagreement>>,
    pub error: Option<StageError>,
    pub tokens_used: Option<u32>,
    pub cost_estimate_usd: Option<f32>,
    pub retry_of: Option<Uuid>,
}

pub enum StageName {
    Detect,        // legacy: route_decision + detection_result
    Parse,         // legacy: parsed_result（含 extraction）
    Review,        // legacy: review_result（獨立 AI sanity check）
    HumanConfirm,  // legacy: needs_user_confirmation → confirmed_result 寫入
    Save,          // 新增：confirmed_result → canonical Property 寫入
}

pub enum StageStatus { Pending, Running, Succeeded, Failed, Skipped }
```

### Legacy 5-JSONB 對應

| Legacy column | 新 stage | 備註 |
|:--|:--|:--|
| `route_decision` | `Detect.output.route_decision` | |
| `detection_result` | `Detect.output.detection` | document_kind 分類 |
| `parsed_result` | `Parse.output` | OCR + structured extraction |
| `review_result` | `Review.output` | issues / sanity flags |
| `confirmed_result` | `HumanConfirm.output` | + 寫到 Property evidence.confirmed |

---

## AuditEvent（新增）

欄位級別 append-only log，補 legacy `audit_logs`（CRUD diff）不足。

```rust
pub struct AuditEvent {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub entity_type: String,        // "property" | "document" | ...
    pub entity_id: Uuid,
    pub field_path: String,         // "address" | "building_area" | "ownership.owners[0].name"
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub source_kind: SourceKind,    // Ai | Human | System | Migration
    pub actor: Option<ActorId>,
    pub stage_id: Option<Uuid>,     // 若是 AI 寫入，反查 stage
    pub note: Option<String>,
}

pub enum SourceKind { Ai, Human, System, Migration }
```

---

## Postgres DDL Examples

### `properties`

```sql
CREATE TABLE properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    owner_actor_id UUID NOT NULL,
    draft_state TEXT NOT NULL CHECK (draft_state IN ('local_only','synced','conflict')),
    kind TEXT NOT NULL CHECK (kind IN ('sale','rental','land_only','commercial')),

    -- evidence-backed (JSONB EvidenceValue<T>)
    address JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    land_number JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    building_no JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    building_area JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    land_area JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    ownership JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    building_age JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    floor_info JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    parking JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    current_loan JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    environment JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,

    -- user input
    display_name TEXT NOT NULL,
    property_type TEXT NOT NULL,
    status TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    notes TEXT NOT NULL DEFAULT '',

    -- sale / rental specific
    sale_price_twd NUMERIC,
    commission_rate NUMERIC,
    monthly_rent_twd NUMERIC,
    deposit_amount_twd NUMERIC,
    lease_term_months INT,

    -- generated columns（從 evidence.confirmed 抽出 index 欄位）
    address_city TEXT GENERATED ALWAYS AS (address #>> '{confirmed,city}') STORED,
    address_district TEXT GENERATED ALWAYS AS (address #>> '{confirmed,district}') STORED,
    land_number_canonical TEXT GENERATED ALWAYS AS (land_number ->> 'confirmed') STORED,

    -- evidence-status consistency check
    CONSTRAINT address_consistent CHECK (
        (address->>'status' = 'confirmed') = (address ? 'confirmed' AND address->'confirmed' IS NOT NULL)
    )
);

CREATE INDEX idx_properties_city ON properties(address_city);
CREATE INDEX idx_properties_district ON properties(address_district);
CREATE INDEX idx_properties_status ON properties(status);
CREATE INDEX idx_properties_kind ON properties(kind);
CREATE INDEX idx_properties_owner ON properties(owner_actor_id);
```

### `documents`

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID REFERENCES properties(id),
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    uploaded_by UUID NOT NULL,
    kind JSONB NOT NULL DEFAULT '{"status":"empty"}'::jsonb,
    original_filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage JSONB NOT NULL,
    current_run UUID,
    version INT NOT NULL DEFAULT 1,
    replaces_document_id UUID REFERENCES documents(id),
    tags TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_documents_property ON documents(property_id);
CREATE INDEX idx_documents_active ON documents(is_active) WHERE is_active;
```

### `processing_runs` + `ai_stages`

```sql
CREATE TABLE processing_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_ids UUID[] NOT NULL,
    property_id UUID REFERENCES properties(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    overall_status TEXT NOT NULL,
    current_phase TEXT,
    error_message TEXT
);

CREATE TABLE ai_stages (
    stage_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES processing_runs(id) ON DELETE CASCADE,
    stage_name TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    provider TEXT,
    model TEXT,
    status TEXT NOT NULL,
    input_ref JSONB NOT NULL,
    output JSONB,
    confidence NUMERIC,
    consensus_metadata JSONB,
    disagreements JSONB,
    error JSONB,
    tokens_used INT,
    cost_estimate_usd NUMERIC,
    retry_of UUID REFERENCES ai_stages(stage_id)
);

CREATE INDEX idx_stages_run ON ai_stages(run_id);
CREATE INDEX idx_stages_status ON ai_stages(status);
```

### `audit_events`

```sql
CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    field_path TEXT NOT NULL,
    old_value JSONB,
    new_value JSONB,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('ai','human','system','migration')),
    actor_id UUID,
    stage_id UUID REFERENCES ai_stages(stage_id),
    note TEXT
);

CREATE INDEX idx_audit_entity ON audit_events(entity_type, entity_id);
CREATE INDEX idx_audit_occurred ON audit_events(occurred_at DESC);
CREATE INDEX idx_audit_stage ON audit_events(stage_id) WHERE stage_id IS NOT NULL;
```

---

## Open Questions

1. **PostGIS 切換時機**（呼應 ADR-006 / audit Open Q #2）：GIS 進入 scope（Phase 3）時起 ADR-008 評估。v1 不採用。
2. **`details JSONB` 後備袋**：legacy 用來塞臨時欄位。新版**禁用** generic JSONB 後備袋；要新欄位走 migration。
3. **`id_number_hash` salt**：建議 server 端統一鹽（不存使用者特定 salt），由 secret 管理。明文身分證**永不入 DB**。
4. **`consensus_metadata` schema**：legacy 未文件化，待 Phase 2 開始實作 multi-model 時定型。
5. **`environment` 80+ 欄位拆解時機**：v1 整包 JSONB；若 UI 需要欄位級 evidence，Phase 3 拆。
6. **`property_investigation_reports.data JSONB`**：legacy 已是「不動產說明書」版本快照。v1 是否獨立 entity？建議 Phase 2 起 ADR 決定。

---

## Next Steps

1. ADR-005 / ADR-006 / data-model-v1 都 ready → 可以開始 scaffold Tauri app
2. scaffold 後第一個 Rust integration test：寫 `EvidenceValue<T>` invariant test（覆蓋上述 6 條規則）
3. scaffold 後第一個 SQL migration：本檔 §Postgres DDL Examples 的四個 table（properties / documents / processing_runs / ai_stages / audit_events）
4. People / GIS / KYC / Transactions 等 domain 進入 scope 時逐個擴 entity（依 PRD Phase 計畫）
