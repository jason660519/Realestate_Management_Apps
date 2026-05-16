# ADR-005: Frontend Stack — React + TypeScript + Mantine + TanStack

Status: Proposed
Date: 2026-05-17
Owner: Project lead
Related: ADR-002, ADR-003, `docs/architecture/frontend-and-db-trade-offs.md`, `DESIGN.md`

## Context

PRD §6.1 規定 UI 是 Tauri WebView 內的 web frontend，但沒鎖框架。`frontend-and-db-trade-offs.md` 已列出 React / Vue / Solid 比較。本 ADR 鎖定組合，並指定相關工具版本，作為 scaffold 直接輸入。

驅動條件：

- Solo lead（無前端團隊分工）
- Dense workbench（重 table、form、PDF + bbox overlay）
- 不需要 SSR（Tauri local bundle）
- Evidence-first UI 需要文件預覽 + 欄位 inline edit + 多模型 disagreement 顯示
- 高度依賴 AI 輔助開發（Claude / ChatGPT）

## Decision

採用以下組合：

| 用途 | 選擇 |
|:--|:--|
| Language | TypeScript（strict mode）|
| Framework | **React 18+** |
| Build tool | **Vite**（Tauri 預設）|
| Component library | **Mantine v7+**（含 hooks）|
| Routing | **TanStack Router**（type-safe routing）|
| Server state / cache | **TanStack Query**（v5）|
| Data table | **TanStack Table**（v8）|
| Form | **React Hook Form + zod** |
| Date | `dayjs`（Mantine 預設）|
| PDF preview | `react-pdf`（pdf.js wrapper）|
| Icons | `@tabler/icons-react`（Mantine 預設）|
| Tauri binding | **tauri-specta**（自動產 TS types from Rust commands）|
| Testing | Vitest + Testing Library + Playwright（E2E）|

## Reasoning

1. **Solo dev × AI 槓桿**
   Tauri 2 + React + TanStack + Mantine 在 2026 年的 Stack Overflow / GitHub / Claude 訓練資料中是最厚的組合。Vue 與 Solid 可做同樣事情，但你跟 AI 對話拿答案的命中率會明顯下降。

2. **Dense workbench 對齊 Mantine**
   Mantine 設計傾向 dense / utility，內建 dark mode、緊湊密度、AppShell（navbar + header + aside），對 DESIGN.md 描述的 dense operational layout 是現成模板。MUI 太「Material」、AntD 太「toB 中國風」、shadcn/ui 要手組太多元件。

3. **TanStack 全家桶協同**
   Router / Query / Table 三者共享設計哲學（type-safe、headless、framework-agnostic），且 Query + Table 是當前資料密集 SaaS 的事實標準。Evidence-first review 需要的「列表 → 詳情 → inline edit → bbox」流程，這套全部能撐。

4. **Evidence-first UI 的 PDF + bbox overlay**
   `react-pdf` + canvas / SVG overlay 範例豐富。Audit §3.1 顯示 legacy `property_documents.parsed_result` 已有 bbox 雛形，新版要把 evidence source 上的 bbox 顯示出來，React 路徑成熟。

5. **Bundle size 對 Tauri 不是顧慮**
   React + Mantine + TanStack 打包約 300-500 KB（gzip 後）。Tauri 直接 bundle 本地檔案，不走網路，這個 size 完全無感。

## Consequences

### 正面

- 元件庫齊備，scaffold 第一週就有可用 AppShell + table + form
- TS types 從 Rust → tauri-specta → frontend 全程對齊，evidence schema 不會兩邊各寫
- TanStack Query 的 cache invalidation 與 ADR-004 的 local SQLite cache 概念協同（先讀 cache → background refetch from server）

### 負面 / 成本

- React 心智模型（hooks、effect、re-render）對新手需學習時間
- TanStack Router 較新，與 React Router v7 並列雙標準，部分教程會混
- React 18 並發特性（transitions、suspense）使用不當會 debug 痛苦

## Dependencies (initial pin)

scaffold 時建議：

```json
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "@mantine/core": "^7.13.0",
    "@mantine/hooks": "^7.13.0",
    "@mantine/form": "^7.13.0",
    "@mantine/dates": "^7.13.0",
    "@mantine/notifications": "^7.13.0",
    "@tabler/icons-react": "^3.0.0",
    "@tanstack/react-router": "^1.50.0",
    "@tanstack/react-query": "^5.50.0",
    "@tanstack/react-table": "^8.20.0",
    "react-hook-form": "^7.52.0",
    "@hookform/resolvers": "^3.9.0",
    "zod": "^3.23.0",
    "react-pdf": "^9.0.0",
    "dayjs": "^1.11.0",
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "@vitejs/plugin-react": "^4.3.0",
    "vitest": "^2.0.0",
    "@testing-library/react": "^16.0.0",
    "@playwright/test": "^1.46.0",
    "tauri-specta": "^2.0.0"
  }
}
```

實際版本以 scaffold 當下最新穩定為準（用 Context7 / npm view 查），但 React **不超過 18.x**（19 RC 暫不採用）。

## Alternatives Considered

| 方案 | 拒絕理由 |
|:--|:--|
| Vue 3 + Element Plus / Naive UI | 完全可做，但 AI 輔助答案密度較低；Solo 開發吃虧。若 lead 個人偏好 Vue 可改 ADR |
| SolidJS | 效能最佳但元件生態小；solo 維護要自己組元件，時間成本高 |
| Svelte 5 | Tauri 範例少；reactivity rune 仍在演進 |
| MUI（取代 Mantine）| 過度 Material 風，與 dense workbench 美學不合 |
| shadcn/ui（取代 Mantine）| 需手組大量基礎元件；對 solo 是時間黑洞 |
| Redux / Zustand 取代 TanStack Query | Server state 與 UI state 分流；TanStack Query 直接吃 server endpoint，不必另做 store。本地 UI state 用 `useState` / `useReducer` 即可 |
| Bun（取代 Node + npm）| Tauri ecosystem 仍以 Node 為主流，pnpm 比 Bun 穩定。Bun 可選但非必要 |

## Open Questions

1. **Routing**：TanStack Router 雖然 type-safe 較強，社群仍小於 React Router。若你已熟 React Router，可改 v7。Scaffold 時做最終定奪。
2. **Tauri binding 是否真的用 tauri-specta**：v2 兼容性與維護狀態待 scaffold 時驗證（用 Context7 查）。若不成熟，退路是手寫 `src/lib/tauri-bindings.ts`。
3. **i18n**：第一版 zh-Hant-TW only，是否要先預埋 `react-i18next`？建議：**不預埋**，等真的有第二語言需求再加（PRD §10 `property_au_details` 已 defer）。
4. **State machine library**（如 XState）：evidence-first 流程有複雜 FSM（stage 1-5 + retry / fail），是否引入 XState？建議：第一版用 `useReducer` + 手寫 reducer，FSM 邏輯放 Rust 那層，前端只反映狀態。

## Verification

scaffold 後驗證：

```bash
npm run typecheck      # TypeScript strict 全過
npm run lint
npm run test
npm run build          # Vite build → dist/，由 tauri 包進 bundle
cargo tauri dev        # 啟動桌面端，AppShell 可見、navigate 通
```

Review checklist：
- [ ] `tsconfig.json` 開 `strict: true`、`noUncheckedIndexedAccess: true`
- [ ] `.eslintrc` 禁 `any`（呼應 CLAUDE.md）
- [ ] Mantine theme 設定 dense + dark mode（呼應 DESIGN.md）
- [ ] Tauri binding 由腳本自動生成，不手寫 invoke wrapper
- [ ] 無 Next.js / SSR-only 套件混入
