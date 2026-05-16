# Frontend Framework + Database：Decision Inputs

Status: Decision Input（不是 ADR，是給你做選擇的分析）
Date: 2026-05-17
Owner: Project lead
Action expected: 你選定後，把結果寫成 ADR-005（frontend）與 ADR-006（database）

---

# Part 1 — Frontend Framework

## 1.1 候選方案

| 方案 | 摘要 |
|:--|:--|
| **React + TypeScript** | 生態最大，元件庫齊備，Tauri 範例最多 |
| **Vue 3 + Composition API** | API 簡潔，dense UI 表達清楚，學習曲線低 |
| **SolidJS** | 效能最佳，bundle 小，reactivity 模型最先進 |

## 1.2 本專案的 UI 特性（決策時要記得）

從 PRD §12 與 DESIGN.md 推導：

- **Dense workbench**（非 marketing site）
- **重 data table**（property list、document list、AI run trace）
- **重 form**（property 編輯、evidence review、settings）
- **PDF / image preview + bbox overlay**（evidence-first UI）
- **可能有 GIS map**（Leaflet / MapLibre）
- **狀態多**：loading / empty / error / degraded / pending / confirmed
- **不需要 SSR**（Tauri 直接 bundle）

## 1.3 三個框架對應分析

### React + TypeScript

| 維度 | 評語 |
|:--|:--|
| Tauri 整合 | 最成熟，官方 quickstart 用 React |
| 元件庫 | MUI、Mantine、AntD、shadcn/ui、Radix — 全都有 dense desktop 風 |
| Data table | TanStack Table（業界標準）、AG-Grid |
| Form | React Hook Form + zod 是事實標準 |
| 文件 / PDF | `react-pdf`、`pdf.js` wrapper 多 |
| 學習曲線 | 中（hooks 心智模型） |
| 風險 | Next.js / SSR 在 Tauri 用不上，但 docs / 教學常混在一起，新手易誤入歧途 |

### Vue 3

| 維度 | 評語 |
|:--|:--|
| Tauri 整合 | 成熟，官方有 template |
| 元件庫 | Element Plus、Naive UI、PrimeVue — 有 dense 風但深度略遜 |
| Data table | Element Plus Table、Naive UI Data Table — 可用，工具不如 TanStack |
| Form | VeeValidate、FormKit — 不錯但生態小 |
| 文件 / PDF | `vue-pdf-embed` 等，數量少於 React |
| 學習曲線 | 低（SFC 直觀） |
| 風險 | TypeScript 整合近年大改善但仍弱於 React；遇到怪題目找答案資源較少 |

### SolidJS

| 維度 | 評語 |
|:--|:--|
| Tauri 整合 | 可，社群有範例但少 |
| 元件庫 | Kobalte（unstyled）、SolidUI、Hope UI — 都還在早期 |
| Data table | TanStack Table（同 React 引擎，支援 Solid adapter） |
| Form | `@modular-forms/solid` — 不錯但小眾 |
| 文件 / PDF | 多數需自包或基於原生 lib |
| 學習曲線 | 低中（JSX + signal） |
| 效能 | 三者最佳，重 render 場景明顯領先 |
| 風險 | 生態小、招聘難、長期維護要自己扛更多 |

## 1.4 我的建議

**React + TypeScript + Mantine + TanStack Query + TanStack Table + React Hook Form + zod**

理由：
1. 房地產 dense workbench + 重 table + 重 form + PDF preview，React 生態剛好對齊
2. Mantine 對 dense desktop 風格很合（hooks 多、不過度設計），且支援暗色 / 緊湊密度
3. TanStack 全家桶在 Tauri 桌面 app 是穩定組合
4. Evidence-first UI 需要 PDF + bbox overlay，這條路 React 範例最多
5. AI Agent / 教學 / Stack Overflow 答案最厚（你跟 Claude / ChatGPT 對話也最受益）

**什麼情況不選 React：**
- 如果你個人偏好 Vue（你寫起來最舒服 → 採用），Vue 3 完全可以做這個專案
- 如果你想練 Solid 並接受生態風險，Solid 在效能上會贏

**搭配建議：**
- Build tool：Vite（Tauri 預設）
- Routing：TanStack Router 或 React Router v7（前者 type-safe 較強，後者範例多）
- State：第一版用 component state + TanStack Query，不上 Redux/Zustand；真的需要再加
- Tauri binding：`tauri-specta` 自動產生 TS types（呼應 ADR-002）

---

# Part 2 — Database

## 2.1 候選方案

| 方案 | 部署位置 | 摘要 |
|:--|:--|:--|
| **A. Supabase Cloud** | Supabase.com | Legacy 在用的方案 |
| **B. Supabase Self-Hosted** | 內網 server | Supabase stack 跑在 192.168.1.6 |
| **C. Plain Postgres + Custom API** | 內網 server | 自跑 postgres + 自寫 typed API |

## 2.2 對齊 PRD 與 ADR 的限制

- **PRD §6.2**：「重型服務在內網 server」→ 排除 **A**（Cloud）
- **PRD §14**：「Secrets are never shown raw / AI provider credentials are app-scoped or server-scoped by documented decision」→ Supabase 的 service_role key 與 cloud 連線都要小心
- **ADR-002**：Server 是 canonical source，desktop 走 HTTP → 不論選 B 或 C，都需要 server-side API

## 2.3 三個方案對應分析

### A. Supabase Cloud

| 維度 | 評語 |
|:--|:--|
| 部署成本 | 0（已有） |
| Legacy migration | 最直接（schema 不動） |
| 內網要求 | **不符** |
| 資料主權 | 雲端，違反 PRD |
| 結論 | **排除**（PRD 不允許） |

### B. Supabase Self-Hosted（完整 stack）

| 維度 | 評語 |
|:--|:--|
| 部署成本 | 中高（8+ container：postgres、auth、storage、realtime、kong、studio、meta、edge） |
| Legacy migration | 平滑（legacy 也是 Supabase） |
| 開箱功能 | PostgREST API、auth、object storage、realtime、Row-Level Security |
| 內網要求 | 符合 |
| 維運 | 升級鏈長（多 container）、需追 Supabase release |
| 適合 | 想保留 RLS / auth / storage / realtime 大部分功能 |

### B'. Supabase Self-Hosted（精簡 stack）

| 維度 | 評語 |
|:--|:--|
| 服務 | postgres + PostgREST + GoTrue (auth, 可選) + Storage (可選) |
| 部署成本 | 中（3-4 container） |
| 取捨 | 拿掉 realtime / studio / edge functions |
| 適合 | 想要 PostgREST 的方便，但不需要 realtime / edge |

### C. Plain Postgres + Custom Typed API

| 維度 | 評語 |
|:--|:--|
| 部署成本 | 低（1 個 postgres container + 1 個 API container） |
| Legacy migration | Schema 可移植，但 RLS policy / auth helper 要重寫 |
| 開發成本 | 高（要自寫 CRUD、auth、storage） |
| 內網要求 | 符合 |
| 控制力 | 最高 |
| API 候選 | Rust axum（與 desktop 同語言生態） / NestJS / FastAPI / Fastify + Prisma |
| 適合 | 用不到 Supabase 多數功能、想要最小依賴、能接受開發初期慢一點 |

## 2.4 我的建議

**先 B'（Supabase Self-Hosted 精簡 stack），保留 C 為長期退路。**

理由：

1. **Legacy schema migration 平滑**
   舊 SPA schema 已在 Supabase 上跑通；先延續可以省 Phase 5 大量功夫。
2. **PostgREST 即拿即用 API**
   不必第一週就自寫 typed API；server 端可以更快有東西可呼叫。
3. **精簡 stack 不背 realtime / edge 的維運**
   只跑 postgres + PostgREST + auth（可選），維運成本接近純 postgres。
4. **未來要切到 C 不難**
   Postgres schema 不變，只是把 PostgREST 換成自寫 API；Phase 4-5 視情況再評估。

**什麼情況直接選 C：**
- 你**完全用不到 Supabase 的 auth / storage / RLS**（單機 single user 情況確實如此）
- 想用 Rust axum 與 desktop 共享 Rust 生態（吸引人但開發成本高）
- 對 Supabase 升級鏈感到不安

**什麼情況反而選 B 完整：**
- 想保留 Supabase Studio 在本機看資料（debug 友善）
- realtime 用得到（例：另一台 desktop 即時看到本機改動）

## 2.5 跟 ADR-006 連動的決定

選定後 ADR-006 要回答：

1. Postgres 版本（建議 16）
2. PostgREST or custom API
3. Auth 怎麼做（單機可省略 / 內網單一憑證 / 完整 GoTrue）
4. Storage 在 Supabase Storage 還是 MinIO 還是 fs + nginx
5. RLS 用不用（單 user 不必，多 user 必開）
6. EvidenceValue 在 schema 是 JSONB column 還是獨立 evidence_values 表

---

# Part 3 — 我給你的二選一摘要

| 題目 | 我推薦 | 退路 | 不選 |
|:--|:--|:--|:--|
| Frontend | React + TS + Mantine + TanStack | Vue 3 + Element Plus | Solid（生態風險） |
| Database | Supabase Self-Hosted（精簡 stack）on 192.168.1.6 | Plain Postgres + Rust axum API | Supabase Cloud（違反 PRD） |

決定完直接告訴我，我會把這份分析中對應的段落收斂成 ADR-005、ADR-006，然後就可以準備 scaffold。
