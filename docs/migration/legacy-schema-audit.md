# Legacy Schema Audit — Owner-Property-Management-AI-SPA

Status: Audit
Date: 2026-05-17
Source: `/Volumes/KLEVV-4T-1/Real Estate Management Projects/Owner-Property-Management-AI-SPA/`
Audience: Rust + Tauri rebuild (本專案) 設計者
Related: `docs/architecture/data-model-v0.md`、`docs/product/prd.md`

---

## 1. Overview

| 項目 | 內容 |
|---|---|
| DB engine | **Supabase Postgres**（PG 17，extensions: `uuid-ossp`、`postgis`、`pg_net`） |
| Source of truth | `supabase/migrations/` 188 個 SQL 檔（含 README、MIGRATION_GUIDE） |
| Baseline | `20260122000000_full_schema.sql`（30 tables）→ 經 187 個增量 migration 演進至 ~170 tables |
| Codegen type file | `packages/shared-types/database.ts`（10590 行；Supabase TS codegen 完整快照） |
| External backends | `backend/elasticsearch/`（people DB 全文搜尋；非 Postgres）；目前無 Prisma / TypeORM / Drizzle |
| RLS posture | 幾乎所有表 `ENABLE ROW LEVEL SECURITY`；`auth.uid()` + IAM（自製 RBAC）+ `service_role` bypass；super_admin 表多採 deny-all + 角色白名單 |
| Migration count | 188（其中 70+ 個含 `CREATE TABLE`，共 209 個 `CREATE TABLE` 語句，含 `IF NOT EXISTS` 重複） |

> 重要：legacy 是 Supabase + Next.js SPA + Edge Functions 架構，許多 RLS / auth.uid() 慣例**不會**直接遷移到 Rust + Tauri + 內網 server。表結構可承襲，但授權邊界會重畫。

---

## 2. Table Inventory（按 domain 分組）

### 2.1 Identity / Auth / Permission（11 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `users_profile` | 使用者主檔（綁定 `auth.users`） | 含 `id_number_enc`（PII 加密）、`region`、`roles[]`、社群連結；新 model 重畫權限後可保留欄位 |
| `iam_roles` | RBAC 角色（含 `parent_role_id` 階層） | 自製 IAM，**非** Supabase auth.roles |
| `iam_role_permissions` | role × resource × actions[] × scope | scope 支援 own/region/all |
| `iam_user_roles` | 使用者-角色多對多 | `assigned_at`、`assigned_by` |
| `iam_groups` / `iam_group_members` / `iam_group_roles` | 群組-角色繼承 | `is_system_managed` 標記系統群組 |
| `roles` / `role_permissions` / `permissions` | 舊版 RBAC（被 iam_* 取代但仍存在） | **drop candidate** |
| `agent_authorizations` | 房東授權仲介管理特定物件 | `permissions JSONB` + `property_ids[]` + `valid_from/until` |
| `landlord_team_access` | 房東團隊成員權限 | |
| `application_role_tags` / `ai_model_role_tags` | role 與 application/AI module 對應 tag | seed-driven |

### 2.2 Property / 物件（13 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `property_sales` | 售屋物件主檔 | **巨型表 ~100 欄**，含結構化地址、面積（坪 / 平方公尺多種）、樓層、車位、銀行貸款、佣金、`details JSONB` 退路 |
| `property_rentals` | 租屋物件主檔 | 同上規模，多了 `monthly_rent`、`deposit_amount`、`rental_deposit_months`、`lease_term` |
| `property_au_details` | 澳洲市場專屬欄位 | `au_` prefix（strata、council、auction、suburb 等 ~40 欄）；國際化雛形 |
| `property_owners` | 物件所有人 + 代理人 | `owner_*`、`proxy_*`、`id_number_enc` |
| `property_photos` | 照片 | `is_primary`、`sort_order`、`photo_type` |
| `property_inventory` | 物件附屬清單（冷氣 / 家具） | `condition` 文字 |
| `property_environment_conditions` | 環境條件 ~80 欄 | 用於「不動產說明書」，含 `area_indoor`、`area_arcade`、`announced_land_value`、`management_fee_*`、`nearby_*`；**多數欄位 OCR 抽取候選** |
| `property_documents` | 物件相關文件（謄本、權狀、合約） | **evidence 旗艦表**（見 §4.1） |
| `property_status_history` | 狀態變更歷程 | `old_status` → `new_status` + `metadata JSONB` + `reason` |
| `property_type_change_logs` | 物件類型/價格變更 | 含 `effective_date` |
| `property_investigation_reports` | 不動產說明書版本 | `version` + `data JSONB`；versioning by row |
| `property_faqs` / `property_comparisons` / `property_agent_assignments` | 衍生關聯 | |
| `building_title_records` | 建物所有權狀（從謄本抽取） | `owner_id_number`、`encumbrances JSONB`、`ocr_extracted` flag、`ocr_parsing_log_id` FK |
| `buildings_communities` | 社區/大樓主檔 | `amenities[]`、`rules_regulations JSONB` |

### 2.3 Document / 文件（11 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `property_documents` | **核心文件表**（含 OCR/VLM 中繼狀態） | 詳見 §4.1 |
| `document_uploads` | 通用文件上傳（非物件專屬） | `related_entity_id` + `related_entity_type` polymorphic |
| `ocr_parsing_logs` | OCR 執行紀錄 | `extracted_text`、`structured_data JSONB`、`confidence_score`、`ocr_engine`、`processed_by` |
| `ocr_parse_results` | OCR 結果（多模型 fan-out） | `raw_output JSONB`、`token_usage`、`role`（unique parser key）、`provider` |
| `local_ocr_parse_results` | 本機 OCR 結果（離線替代） | |
| `vlm_parsing_logs` | Visual LM 解析紀錄 | `extracted_data JSONB`、`confidence_score`、`vlm_model`、`cost` |
| `digital_signatures` / `contract_signatures` | 電子簽章 | |
| `contracts` / `lease_agreements` / `sales_agreements` | 合約紀錄 | |
| `panorama_images` | 360 全景圖 | `hotspots JSONB` |
| `media_gallery` / `media_processing_queue` | 媒體佇列 | |

### 2.4 AI / Processing（22+ tables）

| Table | 用途 | 備註 |
|---|---|---|
| `ai_agent_model_assignments` | AI agent 主/備模型路由 | `primary_provider/model_id` + `fallbacks JSONB` + `guardrails JSONB` |
| `ai_api_keys` | 使用者 AI 廠商 key（AES 加密） | `api_key_encrypted` + `iv`，`is_valid` flag |
| `ai_call_rate_limits` | 呼叫節流計數 | `endpoint_key` + 時序 |
| `ai_chat_logs` | 一般對話紀錄 | `content JSONB`、token、cost、duration |
| `ai_conversations` | 較高層級對話 session | `entities_extracted`、`sentiment_analysis`、`intent_detected` |
| `ai_key_validation_cache` | API key 驗證快取 + 可用模型 | |
| `ai_model_evaluations` | 模型可用性 / 候選清單 | `is_working`、`is_candidate`、`display_status_override` |
| `ai_model_research_reports` | 模型研究報告（自動生成） | `report_markdown`、`source_urls[]`、價格、context window |
| `ai_model_role_assignments` / `ai_model_role_tags` | role × module × model 配置 | |
| `ai_model_selections` | 全域選用模型 | |
| `ai_modules_assigned_function` | module → function 對應 | |
| `ai_performance_metrics` | 模型效能聚合 | |
| `ai_prompt_audit_logs` | Prompt 稽核（含 injection flag） | `injection_flags[]`、`user_input_sha256`、`prompt_source` |
| `ai_settings_validation_summary` | 使用者 AI 設定狀態 | |
| `ai_system_prompts` | System prompt 版本管理 | `version`、`is_active`、`source_saved_prompt_id` |
| `ai_usage_logs` | 通用 AI 使用紀錄（含成本） | `final_prompt_hash`、`prompt_source`、`response_status` |
| `available_ai_models_and_version` | 可用模型清單 seed | |
| `anthropic_credit_guard` | Anthropic 額度 circuit breaker | |
| `saved_prompts` | 使用者儲存的 prompt | |
| `transcript_parse_jobs` | 謄本解析 job 佇列 | `progress JSONB[]`、`payload`、status FSM |
| `transcript_intake_runs` | **謄本攝取 workflow state**（路由→偵測→解析→審查→確認） | `route_decision/detection_result/parsed_result/review_result/confirmed_result` 五段 JSONB；對應新 model 的 ProcessingRun（見 §4.4） |
| `adapter_evaluation_runs` | LLM adapter 評測 | |
| `image_to_image_evaluation_runs` | 圖生圖評測 | `2d_url`、`3d_url` |
| `llm_observability_traces` / `llm_observability_invocations` | OpenTelemetry 風格 trace/span | Langfuse-inspired；workflow trace + invocation span |
| `llm_configs` | adapter 設定 | |

### 2.5 GIS / 地圖（3 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `lvr_land_transactions` | **實價登錄成交資料倉儲** | `latitude/longitude DOUBLE PRECISION`、`land_section_tokens[]`、city/district/village、價格；用於行情查詢 |
| `nearby_facilities` | 物件附近設施 | `distance_meters`、`walking_time_minutes`、`facility_type` |
| `regions_settings` | 地區設定（city/district/region） | |

> **重要**：PostGIS extension 已 install 但**幾乎不用**——所有 GIS 資料是 `lat double / lng double` + B-tree index，沒有 `geometry/geography` 型別。新版若需 spatial query 必須考慮匯入 PostGIS 或在 Rust 端用 RTree。

### 2.6 People Database（11 tables — 人物資料庫 sub-system）

> Sprint 145 引入的獨立 sub-system，從各種來源檔（里長名冊、企業名錄）批次匯入並做 entity resolution。**結構幾乎就是 evidence pattern 的範本**。

| Table | 用途 | 備註 |
|---|---|---|
| `people_records` | 人物記錄（公開介面表） | `ocr_confidence`、`quality_score`、`duplicate_flag`、`duplicate_of_id` |
| `people_duplicates` | 重複候選 + 人工審核 | `similarity_score`、`review_status`、`reviewed_by/at` |
| `import_batches` | 匯入批次 | `status` FSM、`total/processed/skipped_records`、`error_message` |
| `people_db_files` | sha256-key 檔案 inventory | status FSM `pending → parsing → parsed → ocr_queued → normalized → resolved → indexed`（多分支） |
| `people_db_staging_records` | **raw + normalized JSONB 雙欄** | `raw JSONB NOT NULL`（parser 原樣）+ `normalized JSONB`（normalize worker 填）+ `person_id` back-ref |
| `people_db_persons` | 正規人物 canonical entity | `canonical_id_no UNIQUE`、`source_count`、`quality_score` |
| `people_db_person_sources` | staging_records → person 對應 | `match_reason` enum: id_exact / confirmed_name_phone / confirmed_name_addr / new |
| `people_db_merge_candidates` | 模糊比對待審 | `confidence`、`status` pending/confirmed/rejected、`decided_by/at` |
| `people_db_merge_blacklist` | 拒絕配對（避免重複建議） | 唯一 (person_a, record_b) |
| `people_db_ingest_runs` | Orchestrator stage 執行紀錄 | stage enum: scan/parse/normalize/resolve/reindex/all |
| `people_import_jobs` | UI 觸發匯入 job | |
| `people_db_files_ocr_columns` | OCR 結果欄位（patch） | migration patch |
| `identity_verification_records` | 個人身分驗證 | `id_number_encrypted`、`ai_risk_score`、`face_match_score`、`ocr_confidence_score`、`ocr_extracted_data JSONB` |
| `dataset_metadata` | 來源資料集中繼資料 | |

### 2.7 CRM / Leads / Customers（11 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `leads_buyers` / `leads_tenants` | 買方/租方線索 | |
| `buyer_inquiries` / `buyer_intentions` / `tenant_inquiries` | 詢問記錄 | |
| `viewing_appointments_buyer` / `viewing_appointments_tenant` | 看屋預約 | |
| `landlord_customers` | 房東客戶 CRM | |
| `landlord_availability_settings` | 房東可預約時間 | |
| `landlord_call_preferences` | 通話偏好 | |
| `contact_messages` | 公開站詢問訊息 | `source_tracking` |
| `contact_lead_notes` | 線索註記 | |
| `rental_applications` | 租屋申請 | |
| `purchase_offers` | 出價 | |

### 2.8 Transactions / Finance（13 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `contracts` / `lease_agreements` / `sales_agreements` | 合約 | versioning by `version` int |
| `contract_signatures` / `digital_signatures` | 簽章 | |
| `contracted_buyers` / `contracted_tenants` | 締約方 | |
| `earnest_money_receipts` / `deposit_receipts` / `rent_receipts` | 收據 | |
| `payment_transactions` / `payment_workflow` | 支付 | workflow 用 FSM enum |
| `financial_transactions` | 金流總帳 | reporting view 已建 |
| `rental_ledger` / `sales_ledger` | 各物件分類帳 | |
| `bank_accounts` | 收款帳戶 | encrypted |
| `tax_rates` / `tax_reports` / `invoice_records` | 稅務 | |

### 2.9 Tasks / Cross-app Integration（10 tables）

> 這組是「跨 app plugin contract」相關，新版 Tauri 會在 host shell 重畫；多數**不直接移植**。

| Table | 用途 | 備註 |
|---|---|---|
| `paperclip_tasks` | Paperclip 任務 | `worktree_branch/slug`、`assigned_agent`、`adapter_type`、attempt/cooldown |
| `paperclip_task_events` | 任務事件 | |
| `paperclip_cron_configs` | 排程設定 | |
| `paperclip_webhook_logs` | Webhook 事件 queue | FSM pending/processing/processed/failed/skipped |
| `sync_conflicts` | VIS ↔ Roadmap 同步衝突 | `local_value/remote_value JSONB`、`resolution_note` |
| `dev_tasks` | 開發任務 | |
| `engineer_profiles` | 工程師檔案 + 認領設定 | `hourly_rate`、`max_concurrent_tasks` |
| `todo_tasks` / `tutorial_progress` | 一般待辦 | |
| `user_integrations` | 使用者第三方整合（Google/LINE 等） | `tokens` 加密 |
| `email_threads` / `messages` | Email 整合 | |

### 2.10 Audit / Events / Logs（13 tables）

| Table | 用途 | 備註 |
|---|---|---|
| `audit_logs` | **通用稽核 log**（CRUD diff） | `old_data/new_data JSONB`、`ip_address`、`user_agent`、`resource_table/id` |
| `rbac_audit_logs` | RBAC 變更稽核 | |
| `ai_prompt_audit_logs` | Prompt 稽核 | injection flags、SHA256 |
| `behavior_logs` / `behavior_daily_stats` | 行為追蹤 + cleanup function | |
| `api_call_logs` / `call_logs` | API/通話 | |
| `error_logs` | 錯誤 | |
| `logs` | 通用 logs | |
| `user_activity_logs` / `users_track_history` | 使用者行為歷史 | |
| `system_maintenance_logs` | 維護 log | |
| `backup_run_logs` / `backup_restore_logs` | 備份/還原 log | `cloud_result JSONB`、`destinations[]` |
| `version_history` | 任意實體版本快照 | |

### 2.11 Misc（其餘 ~35 tables）

簡述（不展開）：
- 通知：`system_notifications`、`notification_preferences`、`notification_queue`、`notification_templates`
- 內容：`blog_posts`、`blog_platform_posts`、`blog_analytics`、`saved_prompts`、`seo_configs`
- 設定：`system_settings`、`platform_settings`、`theme_settings`、`user_page_settings`、`storage_quotas`、`storage_alerts`
- 評論：`user_reviews`、`user_feedback`、`recommendation_logs`、`user_favorites`
- 雜項：`maintenance_requests`、`maintenance_quotes`、`maintenance_vendors`、`agent_directory`、`interior_designers`、`escrow_legal_services`、`service_providers`、`insurance_plans`、`currencies`、`exchange_rates`、`glossary_terms`、`whitelist_blacklist`、`superadmin_blacklist`、`form_drafts`、`draft_autosave`、`transfer_tokens`、`email_verifications`、`virtual_phone_numbers`、`social_auth_connections`、`comfyui_styles`、`elasticsearch_indices`、`cloud_resources_monitoring`、`web_vitals`、`perf_metrics`、`web_analytics`、`upload_progress`、`webhook_configs`、`rate_limit_configs`、`user_sessions`、`user_invitations`、`unit_conversion_logs`
- 視圖：`properties`（sales + rentals UNION）、`users_profile_with_role`、`iam_users_view`、`behavior_daily_stats`、`web_vitals_page_summary`

---

## 3. 重點表詳述（Top 6）

### 3.1 `property_documents`（**evidence 旗艦表**，~30 欄）

```sql
-- 摘錄核心欄位（完整 schema 見 packages/shared-types/database.ts:6049）
id UUID PRIMARY KEY
owner_id, uploaded_by UUID REFERENCES users_profile

-- File metadata
file_path, original_filename, mime_type, file_size_bytes
document_name, document_type, property_type
document_date, expiry_date

-- OCR / VLM stage
ocr_status TEXT            -- FSM
ocr_engine TEXT
ocr_confidence_score NUMERIC
ocr_parsing_log_id UUID    -- FK to ocr_parsing_logs
ocr_processed_at TIMESTAMPTZ
ocr_result_path TEXT
vlm_provider TEXT
vlm_model_version TEXT
parse_strategy TEXT
parsing_duration_ms INT

-- Parsed result (canonical from AI)
parsed_at TIMESTAMPTZ
parsed_result JSONB
consensus_metadata JSONB   -- multi-model consensus voting
confidence_score NUMERIC

-- Human verification (evidence-style!)
is_verified BOOLEAN
verified_at TIMESTAMPTZ
verified_by UUID REFERENCES users_profile
verification_notes TEXT

-- Versioning
version INT
replaces_document_id UUID REFERENCES property_documents  -- linked list

-- Misc
used_user_key BOOLEAN      -- 用使用者自帶 key
is_active, tags TEXT[]
```

> **這表已內建半套 evidence pattern**：raw → ocr → parsed → verified 四階段；`consensus_metadata` 甚至支援多模型共識。新版 `EvidenceValue<T>` 可視為這個概念的**正規化抽取**。

### 3.2 `property_rentals` / `property_sales`（~100 欄）

Flat schema + `details JSONB` 退路。地址欄位**已正規化**（`address_city/district/street/number/floor/unit` + `postal_code`），但保留 raw `address`。面積有 11 種維度（`area_main_building/auxiliary/common/extension/parking/...`）。

關鍵 evidence-relevant 欄位：
- `data_source TEXT`（'manual' / 'transcript_parse' / 'lvr_import' / ...）
- `is_data_open BOOLEAN`、`is_investigation_signed BOOLEAN`
- 多個欄位（如 `building_age_years`、`current_loan_amount`）顯然來自謄本 OCR，但表上**沒有 confidence/source 對應**——這就是新版要加 `EvidenceValue<T>` 的點。

### 3.3 `transcript_intake_runs`（**workflow FSM**）

```sql
property_id, property_type ('sale'|'rental')
source_document_ids UUID[]
status FSM: uploaded → route_selected → detecting → parsing
        → reviewing → needs_user_confirmation → confirmed | failed
current_phase TEXT
route_decision JSONB     -- 技術路由（local Python / VLM / structured JSON）
detection_result JSONB   -- AI 案件分類 + 證據
parsed_result JSONB      -- parsing 結果
review_result JSONB      -- 獨立 AI review，含 issues
confirmed_result JSONB   -- 使用者確認後 canonical
error_message
```

> **這就是 `ProcessingRun` / `AIStageTrace` 的 prior art**。新版的 `AIStageTrace` 可視為把這五個 JSONB 欄位拆成正規 stages 表。

### 3.4 `people_db_staging_records`（**raw + normalized 雙欄範本**）

```sql
id, file_id REFERENCES people_db_files
record_index INT (file 內序號)
raw JSONB NOT NULL          -- parser 原樣 (Record<string,string>)
normalized JSONB            -- normalize worker 寫入 (E.164、ROC→CE 等)
normalized_at TIMESTAMPTZ
resolved_at TIMESTAMPTZ
person_id UUID              -- ER worker 寫入 canonical 對應
```

GIN index on `normalized` 支援 (name+phone)、(name+addr) 模糊比對。**state machine 在 sibling `people_db_files.status` 欄位上**——這是個值得學習的解耦（檔案級 vs row 級 status）。

### 3.5 `identity_verification_records`

身分驗證集大成：
```sql
full_name, date_of_birth, id_number_encrypted, address, city/district/postal_code
document_front_path, document_back_path, selfie_path
face_match_score NUMERIC
ai_risk_score NUMERIC
ai_flags JSONB
ocr_confidence_score NUMERIC
ocr_extracted_data JSONB
device_info JSONB, ip_address INET
status, submitted_at, reviewed_by, reviewed_at, approved_at, rejection_reason
```

> 同時擁有「AI extracted」+「AI risk」+「human review」三段；新版若做 KYC 可直接套 `EvidenceValue<T>`。

### 3.6 `llm_observability_traces` + `llm_observability_invocations`

```sql
-- traces (workflow level)
trace_key UNIQUE
user_id, page_path, module_key, invocation_name, execution_name
status: running/success/error/timeout/cancelled
started_at, ended_at, metadata JSONB

-- invocations (span level)
trace_id REFERENCES traces
source_kind: llm_call/adapter_run/evaluator_run/tool_call/legacy_usage
provider, adapter_id, adapter_model, requested_model, effective_model
input_prompt, raw_output, rendered_output
tokens_input/output, cost_usd, ttft_ms, e2e_ms, throughput_tokens_per_s
evaluation_label/score/message
```

> Langfuse 風格 trace+span。新版 `AIStageTrace` 可借此結構，但簡化為單表（trace_id + stage_index）。

---

## 4. Evidence Pattern Audit

### 4.1 已存在類似 source/extracted/edited/confirmed 結構的表

| Table | 證據欄位 | 缺漏 |
|---|---|---|
| `property_documents` | `parsed_result`（AI）、`consensus_metadata`（多模型共識）、`verified_*`（human confirm）、`confidence_score`、`ocr_confidence_score` | 無 bbox/page locator、無 `human_edited` 中間態（直接 verified）、無欄位級別追蹤（檔案級） |
| `identity_verification_records` | `ocr_extracted_data`、`ocr_confidence_score`、`ai_risk_score`、`ai_flags`、`reviewed_by/at`、`approved_at`、`rejection_reason` | 無 `human_edited`、無 stage trace ref |
| `transcript_intake_runs` | `route_decision/detection_result/parsed_result/review_result/confirmed_result`（5 階段 JSONB） | 流程級 evidence，**欄位級**沒有；下游寫入 `property_rentals/sales` 時就遺失 lineage |
| `people_db_staging_records` | `raw JSONB`（parser）+ `normalized JSONB`（worker） | 沒有 confirmed/edited 階段，merge candidates 才有 admin decision |
| `people_records` | `ocr_confidence`、`quality_score`、`duplicate_flag` | flat fields, no per-field provenance |
| `building_title_records` | `ocr_extracted BOOLEAN`、`ocr_parsing_log_id` FK | 整列 boolean，無欄位粒度 |
| `nearby_facilities` | `is_verified BOOLEAN` | 整列 boolean |
| `document_uploads` | `is_verified`、`verified_at`、`verified_by` | flat |
| `ocr_parsing_logs` | `confidence_score`、`extracted_text`、`structured_data`、`status` | log 表，非主資料 |
| `vlm_parsing_logs` | `confidence_score`、`extracted_data`、`cost` | 同上 |

### 4.2 OCR / Confidence 欄位散落

`ocr_confidence_score`、`confidence_score`、`ocr_engine`、`vlm_provider`、`vlm_model_version` 出現在：
- `property_documents`、`identity_verification_records`、`ocr_parsing_logs`、`vlm_parsing_logs`、`people_records`、`building_title_records`

**反覆出現的相同欄位群**，這正是新 `EvidenceValue<T>` 要 DRY 起來的證據。

### 4.3 Audit Trail 現況

- 通用：`audit_logs`（CRUD diff）、`rbac_audit_logs`、`ai_prompt_audit_logs`、`backup_run_logs`
- 物件層級：`property_status_history`、`property_type_change_logs`、`version_history`
- 缺：**沒有欄位級別**的 `audit_event`（新版要新增）

### 4.4 結論

Legacy **已做到的**：
1. 檔案/列級 OCR confidence + verified flag
2. Workflow 級 multi-stage JSONB（transcript_intake_runs 五段）
3. Raw + normalized 雙欄（people_db_staging_records）
4. 多模型 consensus（`consensus_metadata`）
5. LLM observability traces

Legacy **沒做到的**（新 model 要補）：
1. **欄位級** evidence wrapper（每個 property 欄位都有 ai/edited/confirmed 三態）
2. Source locator（page + bbox + raw_text）反查文件
3. Confirmed 即 canonical 的硬規則（目前散落多表）
4. AI 不可寫 confirmed 的 invariant（程式碼層保證）
5. 統一 `audit_event` append-only log

---

## 5. Migration Considerations（per major table）

> 標記說明：**direct map**（直接搬）/ **needs evidence wrapper**（包 `EvidenceValue<T>`）/ **drop**（不搬）/ **reshape**（拆/合）/ **defer**（v0 不做）

### 5.1 Property domain

| Legacy table | 動作 | 備註 |
|---|---|---|
| `property_sales` + `property_rentals` | **reshape** | 合併為單一 `Property` + `kind` enum；100 欄拆 group（基本/面積/車位/環境/財務）；evidence-relevant 欄位包 wrapper |
| `property_au_details` | **defer** | v0 只做台灣；保留欄位 schema 供未來 i18n |
| `property_owners` | **needs evidence wrapper** | 所有人姓名/身分證/地址都應 evidence-wrap（PII + 由謄本 OCR）|
| `property_photos` | **direct map** | 純檔案中繼資料 |
| `property_environment_conditions` | **needs evidence wrapper** | ~80 欄多數來自 OCR 抽取；最大宗 evidence 候選 |
| `property_documents` | **reshape** | 對應新 `Document`；現有 OCR 階段欄位拆到 `ProcessingRun` |
| `property_status_history` / `property_type_change_logs` | **direct map** | 已是 append-only audit |
| `property_investigation_reports` | **direct map** | `data JSONB` + version；契合「不動產說明書」PRD 流程 |
| `building_title_records` | **needs evidence wrapper** | 謄本抽取的標的 |
| `buildings_communities` | **direct map** | |

### 5.2 Document / AI

| Legacy table | 動作 | 備註 |
|---|---|---|
| `property_documents` | **reshape** → `Document` | OCR stage 欄位獨立到 `ProcessingRun` |
| `ocr_parsing_logs` / `ocr_parse_results` / `vlm_parsing_logs` / `local_ocr_parse_results` | **reshape** | 合併為 `ai_stage_trace`（per stage row）|
| `transcript_intake_runs` | **reshape** → `ProcessingRun` | 五段 JSONB 拆成 stages 表 |
| `transcript_parse_jobs` | **reshape** | 整合進 `ProcessingRun` 的 status FSM |
| `ai_agent_model_assignments` / `ai_model_role_assignments` / `ai_model_selections` | **direct map** | 設定資料 |
| `ai_system_prompts` / `saved_prompts` | **direct map** | |
| `ai_api_keys` / `user_vlm_credentials` | **reshape** | 桌面版改為本機 keychain 儲存，不入 DB |
| `ai_usage_logs` / `ai_chat_logs` / `ai_conversations` / `ai_prompt_audit_logs` | **direct map** | 用量/稽核照搬 |
| `ai_call_rate_limits` / `anthropic_credit_guard` | **defer** | 桌面版單機優先；後續 server 端再做 |
| `ai_model_evaluations` / `ai_model_research_reports` / `adapter_evaluation_runs` / `image_to_image_evaluation_runs` | **defer** | superadmin 域，v0 不做 |
| `llm_observability_traces` / `llm_observability_invocations` | **reshape** | 與新 `AIStageTrace` 合併 |
| `consensus_metadata`（欄位） | **direct map** | 保留多模型共識 metadata |

### 5.3 People

| Legacy table | 動作 | 備註 |
|---|---|---|
| `people_db_files` / `people_db_staging_records` / `people_db_persons` / `people_db_person_sources` / `people_db_merge_candidates` / `people_db_merge_blacklist` / `people_db_ingest_runs` / `people_import_jobs` | **defer** | PRD §10 列 People 但 v0 不做；保留 schema 供 Phase 2+ |
| `people_records` / `people_duplicates` / `import_batches` | **defer** | 同上 |
| `identity_verification_records` | **needs evidence wrapper** | 若 v0 含 KYC，這是 evidence pattern 最佳套用點 |

### 5.4 GIS

| Legacy table | 動作 | 備註 |
|---|---|---|
| `lvr_land_transactions` | **direct map** | 行情查詢必備；考慮 server 端用 PostGIS geom 取代純 lat/lng |
| `nearby_facilities` | **direct map** | |
| `regions_settings` | **direct map** | |

### 5.5 CRM / Leads / Transactions / Tasks

| Domain | 動作 | 備註 |
|---|---|---|
| Leads + Inquiries + Viewing | **direct map** | 純 CRUD |
| Contracts + Receipts + Ledgers | **direct map** | 金流照搬 |
| `paperclip_*` / `sync_conflicts` / `engineer_profiles` / `dev_tasks` | **drop** | 跨 app dev workflow，新版有獨立 plugin contract |
| `email_threads` / `messages` / `user_integrations` | **defer** | v0 桌面版不整合 email |

### 5.6 Identity / IAM

| Legacy table | 動作 | 備註 |
|---|---|---|
| `users_profile` | **reshape** | 桌面版單機優先，不需 Supabase auth；以 server 端 IAM 替代 |
| `iam_roles` / `iam_user_roles` / `iam_groups` / `iam_*` | **reshape** | 簡化為 server-side RBAC；不直接複用 |
| 舊版 `roles` / `permissions` / `role_permissions` | **drop** | 已被 iam_* 取代 |
| `agent_authorizations` | **direct map** | 仲介授權邏輯保留 |

### 5.7 Audit / Logs

| Legacy table | 動作 | 備註 |
|---|---|---|
| `audit_logs` | **reshape** | 改為新 `audit_event` append-only，欄位級粒度 |
| `property_status_history` / `version_history` | **direct map** | |
| 各種 `*_logs` | **direct map / defer** | 視子系統決定 |

### 5.8 Misc

多數 misc 表（blog、notification、theme、storage_quotas、blacklist、currencies）採 **direct map** 或 **defer**，v0 桌面版不一定需要。

---

## 6. Open Questions（需人工確認）

1. **多 region / 國際化保留多深？** `property_au_details` + `regions_settings` + `users_profile.region/currency` 是雙國雛形。Tauri 版 v0 是否只做台灣？若是則 `region`/`currency` 欄位是否 drop？
2. **PostGIS 真要不要？** Legacy install 了但只用 `double precision` lat/lng + B-tree。新版若要做半徑查詢，是 server 端用 PostGIS 還是 Rust 端 RTree？影響 schema 設計。
3. **`consensus_metadata` 多模型共識細節**？schema 沒看到，需要從 application code 反查；新版 `AIStageTrace` 是否要原生支援 consensus？
4. **`details JSONB` 漏斗欄位是否清理？** `property_sales/rentals.details JSONB` 是過去的 ad-hoc 欄位袋；migration 後是否硬性要求拆出？影響 import 策略。
5. **`agent_authorizations.permissions JSONB` schema**？沒看到正規 schema，需要從 application code 反查。新版若保留授權邏輯需確認結構。
6. **People DB 是否在 v0 就要重建？** Sprint 145 累積大量 entity resolution / 模糊比對基礎建設，v0 是 defer 還是承襲？PRD §10 提到 People 但沒講優先序。
7. **Cross-app plugin contract（saydo / project-manager / paperclip）**：legacy 是直接寫進同 DB（`paperclip_tasks`、`sync_conflicts`），新版 PRD 提到「跨 app plugin contract」需三方同步。schema 邊界要重畫？
8. **`users_profile.id_number_enc`、`property_owners.owner_id_number_enc` 加密方式**？migration 沒寫，可能用 Supabase Vault 或 application-level AES。桌面版要繼承還是改 OS keychain？
9. **`workflow_control` 欄位**（在 `property_rentals/sales`）：用途為何？看似工作流節點，但 schema 沒 enum 約束。需從 application code 反查。
10. **`transcript_intake_runs.confirmed_result JSONB` 寫入 property 表的對應路徑**？目前 schema 沒看到 trigger，可能是 application code 解構。新版若用 `EvidenceValue<T>` 需重設這條 commit path。

---

## 7. Numbers at a Glance

- **Total tables in TS codegen**: ~170（不含 views/RPCs）
- **CREATE TABLE migrations**: 70 個檔案，209 statements（含 IF NOT EXISTS 重複）
- **Tables with `confidence_score` 類欄位**: 7+（property_documents, identity_verification_records, ocr_parsing_logs, vlm_parsing_logs, people_records, ocr_parse_results 等）
- **Tables with `is_verified` / `verified_*`**: 4+（property_documents, document_uploads, nearby_facilities, identity_verification_records）
- **JSONB 欄位數**: 100+（廣泛用於 raw extracted / metadata / config）
- **RLS-enabled tables**: 幾乎全部
- **PostGIS usage**: install 了但實際只用 lat/lng double（無 geometry 欄位）
- **Views**: 5（properties、users_profile_with_role、iam_users_view、behavior_daily_stats、web_vitals_page_summary）
- **RPC functions**: 20+（has_role、check_user_permission、get_*_count 等）

---

## 8. References

- Legacy migrations: `/Volumes/KLEVV-4T-1/Real Estate Management Projects/Owner-Property-Management-AI-SPA/supabase/migrations/`
  - Baseline: `20260122000000_full_schema.sql`
  - People DB sub-system: `20260412140000_*` ~ `20260419*`
  - Transcript workflow: `20260404130000_*`、`20260427100000_*` ~ `20260429*`
  - AI observability: `20260424100000_create_llm_observability_traces.sql`
- Legacy codegen: `packages/shared-types/database.ts`（10590 行）
- New data model: `docs/architecture/data-model-v0.md`
- PRD: `docs/product/prd.md` §10、§11
