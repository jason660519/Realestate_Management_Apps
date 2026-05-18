# Dev Log: Phase 2 Property Frontend Surface

Date: 2026-05-18

延續同日 `dev-property-readonly-2026-05-18.md`（backend 完成 read-only
list_property_summaries），本場接 frontend、補 component test harness，並把
Phase 2 第一刀的 end-to-end 視覺路徑接通。

## Completed

### 1. 前端測試 helper

- 新增 `src/test/render.tsx`：`renderWithProviders` 包 Mantine + React
  Query（test 用 QueryClient 關 retry、零 gcTime / staleTime 確保隔離）。
- 新增 `src/test/mockTauri.ts`：
  - `invokeMock`：替代 `@tauri-apps/api/core` 的 `invoke`，依命令名分派 handler；未註冊命令 throw 出錯訊息避免靜默回 undefined。
  - `installTauriRuntime()` / `uninstallTauriRuntime()`：把 `window.__TAURI_INTERNALS__` 設成 truthy，讓 `isTauriRuntime()` 走 invoke 路徑而非 preview fallback。
  - `setInvokeHandlers(map)`：常數或 function fixture 都接受；回傳 reset。
- 更新 `src/test/setup.ts`：jsdom 沒實作 `window.matchMedia`，Mantine 啟動會炸 — 加 no-match polyfill。
- 既有 14 tests 不受影響；harness 約定可重用於後續 component test。

### 2. `api/tauri.ts` 加 property surface

- `PropertyKind`：`'sale' | 'rental' | 'land_only' | 'commercial' | 'unknown'`。
- `PropertyStatus`：`'draft' | 'active' | 'pending' | 'archived' | 'unknown'`。
- `PropertySummary` 用 **snake_case** field name（PostgREST 直連 read 路徑例外）；其他 typed surfaces 仍 camelCase。Boundary doc 已補命名慣例例外段。
- `listPropertySummaries()`：preview-mode 回兩筆假資料（顯示「Preview · …」標籤，避免被誤認真實資料）；desktop runtime 直接 `invoke('list_property_summaries')`，**不**做 `.catch() => fallback`，錯誤要透到 UI 讓 useQuery `isError` 帶人類確認動作。

### 3. `routes/properties.tsx` 改寫為五態 list view

- `PropertiesPage`（route 入口）只負責從 `useAppData()` 拿 health，計算 `serverConfigured` 後委派給 `PropertiesView`。
- `PropertiesView`（純元件，props: `serverConfigured`）內含 `useQuery({ queryKey: ['property-summaries'], queryFn: listPropertySummaries })`，依狀態走 5 條分支：
  1. `isPending` → Skeleton 三行。
  2. `isError` → Alert，含 reason、preserved 資訊、Retry button。
  3. `data.length === 0 && !serverConfigured` → `EmptyOperationalState` 「Server URL is not configured」。
  4. `data.length === 0 && serverConfigured` → `EmptyOperationalState`「No properties yet」。
  5. `data.length > 0` → `PropertiesTable` Mantine Table，5 欄（Status badge / Display name / Kind badge / Address / Updated 時間）。
- 包含 Refresh 按鈕（`PageHeader` slot），按下走 `query.refetch()`，期間顯示 loading。
- Kind / Status enum 轉 label 用 readonly `Record<EnumType, string>`，避免 fallback 拼字漂移。
- Status badge 顏色：active=green、draft=gray、pending=yellow、archived=dark、unknown=red（紅色，與 evidence-first「不可靜默隱藏 unknown」原則一致）。

### 4. Component tests（5 個）

- `src/routes/properties.test.tsx`：
  - `vi.mock('@tauri-apps/api/core', async () => ({ invoke: invokeMock }))` 頂層注入 mock。
  - 覆蓋全部 5 態（list 渲染、not_configured 空、empty 空、error + retry、unknown kind 不被靜默吃掉）。
  - Retry case 用 `userEvent.click` 模擬人類操作，並交換 fixture 確認重抓成功。

## Verification

- `npm run typecheck`：通過。
- `npm run test`：19 passed / 19（既有 14 + 新 5）。
- `npm run build`：通過，bundle 增加 ~34KB JS（屬於新 page + Mantine Table tree-shake 後）。
- `cargo test`：13 passed / 13（backend 未動，回歸跑過）。
- `npm run tauri:build`：未重跑（無 backend 改動）。

## Technical Notes

- React Query 在 test QueryClient 設 `retry: false`、`gcTime/staleTime: 0`：jsdom 環境內預設 retry 會讓 isError case 等很久；零 cache time 讓每個 test 用 fresh client，不需手動 invalidateQueries。
- `installTauriRuntime` 用 `Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable, writable })`：jsdom window 是 read-only 對 unknown property，必須 defineProperty 才能塞。
- `useQuery` 的 generic 在 vitest type-check 嚴格模式下需顯式給 type；本次用 `useQuery<PropertySummary[]>` 透過 PropertiesQuery type alias 避免重複。
- `errorMessage(unknown)` 三段 narrowing（Error / string / `{ message }`）：Tauri 透回來的 error 是 `AppError` serde tagged enum（`{kind, message}`），命中第三個分支。
- `PropertiesView` 不直接讀 `useAppData`：方便 component test 注入 prop。`PropertiesPage` 把 shell-level 依賴限定在 route 入口。

## Open Questions

1. **PostgREST 上線後是否補 401/403 narrowing？** 目前 `isError` 一律走 generic message；當 server 帶 RLS 後可加分支 `if (status === 401) → 「Server says unauthorized」`。
2. **Refresh polling**：目前只手動 refresh。若需自動，可在 `useQuery` 加 `refetchInterval`（health check 已 30s polling，可拉相同節奏）。延後到性能驗證後再加。
3. **PropertySummary snake_case 例外何時收回**：boundary doc 註腳寫了 v1 read path 走 Rust axum 後可換 camelCase；屆時 frontend 對應改名。

## Next Priority

- 嘗試在實機 desktop 啟動（`dev-start.command` 或重新 `tauri:build`）視覺驗收 Properties 頁五態，特別是 not_configured / error / list 三種狀態的版面。
- SQLite cache 第一張表 `property_summary_cache`（ADR-004 規劃），搭配 `ServerClient::get_json` 成功後寫 cache、offline 時讀並標 stale。
- ADR 草稿：Phase 2 write path（save / confirm_property_field 走 Rust axum service_role），把 ADR-006 service_role 段具體化。
- 三筆 untracked 連續四個 session 沒動 — 強建議下一輪先處理：`.project-manager.json` 加進 `.gitignore`，`docs/dev-logs/.gitkeep` / `docs/features/.gitkeep` 刪除（路徑跟專案規則衝突）。
