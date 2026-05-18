# Dev Log: Phase 2 Foundations

Date: 2026-05-18

延續同日上半場 `dev-design-overhaul-2026-05-18.md`，本場開三條 foundation：資料邊界文件、前端測試 harness、production bundle ADR。

## Completed

### 1. Property / Document 資料邊界規格

- 新增 `docs/architecture/property-document-boundary.md`，把 ADR-002/004/006 套到 property + document 兩個具體 domain。
- 內容：source-of-truth 矩陣、Tauri invoke surface 對照表、HTTP surface 對照表、failure mode 表、v0 → v1 演進、open questions、驗證 checklist。
- 編號決策：此文件以 specification 形式存在，非新 ADR；ADR 編號保留給 auth (ADR-007)、PostGIS (ADR-008)、bundle (ADR-009)。
- 更新 `docs/architecture/README.md` 新增 Specifications 段落。

### 2. 前端測試 harness（Vitest + Testing Library）

- 安裝 `jsdom@^25`、`@testing-library/react@^16`、`@testing-library/jest-dom@^6`、`@testing-library/user-event@^14`。
- 整合 vitest config 進 `vite.config.ts`（`test:` block，jsdom 環境，setupFiles 指向 `src/test/setup.ts`）。
- 新增 `src/test/setup.ts`：註冊 jest-dom matchers、每個 test 後 cleanup。
- 新增 `src/test/README.md`：harness 約定、Mantine wrapper / Tauri mock 路線。
- 第一支純函式 smoke test：`src/lib/status.test.ts`（10 個 case，覆蓋 healthy / warning / disabled / fallback）。
- 第二支資料 sanity test：`src/components/shell/navigation.test.ts`（4 個 case，覆蓋順序、hint、唯一性）。
- 新增 npm scripts：`test`（一次性，CI-friendly）、`test:watch`。
- 更新 `README.md` Verification 段，把 `npm run test` 納入順序。
- 全 14 個測試通過；`npm run typecheck` 也通過。

### 3. ADR-009 Production Bundle and Signing

- 新增 `docs/architecture/ADR-009-production-bundle-and-signing.md`。
- 決策：v0 只出 macOS `.app` + `.dmg`、unsigned；v1 加 Developer ID + entitlements；v1.5 加 notarization + Tauri updater plugin；Linux / Windows 留 Phase 3。
- `tauri.conf.json` 把 `bundle.targets: "all"` 改為 `["app", "dmg"]`，顯式列出避免誤解。
- 列出 v0 release 流程、bundle identifier / version 管理慣例。
- 起 `npm run tauri:build` 驗證 unsigned bundle 能否產出（背景跑，結果見 commit message 或下次 dev log 補記）。

## Verification

- `npm run typecheck`：通過。
- `npm run test`：14 個測試全綠（Vitest v4.1.6，jsdom）。
- `npm run tauri:build`：成功，Rust release 編譯 1m50s。產出：
  - `src-tauri/target/release/bundle/macos/Realestate Management Apps.app`
  - `src-tauri/target/release/bundle/dmg/Realestate Management Apps_0.1.0_aarch64.dmg`（4.1MB，unsigned，aarch64）
  - 還沒實際 mount `.dmg` 與拖到 Applications 驗收（屬於下次開發者要做的視覺驗收）。

## Technical Notes

- `property-document-boundary.md` 明確規定 WebView 不可直接 `fetch('/api/...')`，全部 HTTP 走 Rust `services/server_client.rs`。這條會影響 Phase 2 第一刀的實作順序：先把 `commands/property.rs` 與 `services/server_client.rs` 骨架立起來，UI 再開始打 invoke。
- Vitest config 寫進 `vite.config.ts` 而非另開 `vitest.config.ts`，以便 vite plugin / alias / env 一處維護。`/// <reference types="vitest/config" />` 提供 type augmentation。
- Test 全部用 explicit `import { describe, expect, it } from 'vitest'`，不開 `globals: true`，避免污染全域 namespace 與 TS lib 設定。
- ADR-009 v0 unsigned 策略呼應 ADR-004 Open Q3（macOS Keychain prompt）：v0 階段 secret 落地策略需配合 unsigned 開發 build 行為，正式簽章後 keychain 不會每次彈窗。

## Next Priority

- 驗收剛產出的 `.dmg`：實際 mount、拖到 Applications、right-click → Open（unsigned 首次需略過 Gatekeeper），確認 app 啟動 + config 路徑 + 圖示 + storage diagnostics。
- Phase 2 v0 第一刀：`commands/property.rs` 骨架 + `services/server_client.rs` HTTP 出口集中，先做 read-only `list properties` 走 PostgREST GET。
- 把 React component 第一支 smoke test 加上（需要 `src/test/render.tsx` Mantine wrapper helper 與 `src/test/mockTauri.ts`）。
- Untracked 三筆（`.project-manager.json`、`docs/dev-logs/.gitkeep`、`docs/features/.gitkeep`）待開發者裁示是否 commit / ignore / 刪除。
