# Dev Log: Design Overhaul 與 Settings Doctor 修復

Date: 2026-05-18

## Completed

- 修正 `/doctor` 在 `.claude/settings.json` 報的三條 WebFetch 權限格式（URL → `domain:hostname`）。
- 修正 `.claude/settings.json` 的 PreToolUse hook：把 hooks 陣列項由字串改為 `{ type: "command", command: ... }` 物件結構。
- 把 `src/components/shell/ShellLayout.tsx` 從 Mantine `AppShell` 改寫為自製 CSS grid：68px icon rail、sticky topbar、寬螢幕（≥1200px）右側 panel。
- 新增 `docs/design/shared-ai-desktop-style.md` 作為 Realestate / SayDo / Project-Manager 桌面家族共用視覺基準。
- 擴寫 `DESIGN.md`：補完 shell pattern、資訊架構、視覺 token、元件規範、evidence-first UI、文案規則、acceptance checklist。
- 在 `src/styles.css` 導入 CSS variables token（`--app-bg`、`--rail-bg`、`--panel-bg`、`--panel-border`、`--text-strong`、`--text-muted`、`--amber-accent`、`--active-blue`、`--success`、`--danger`），取代原本 `color-mix(in srgb, ...)` 寫法。
- `navigation.ts` 加上 `NavigationItem` 型別與 `hint` 欄位，作為 rail tooltip 來源。
- `StatusPanel` / `PluginPanel` 把 Mantine `Card` 替換為共用 `.surface` 樣式以對齊新 token。
- 上一個 session 留下的 `dev-start.command` 一鍵啟動腳本與其 README 連動、dev log 一併納入版本控制。

## Verification

- `npm run typecheck` 通過。
- `npm run build`（含 tsc + vite build）通過，輸出 `dist/assets/index-*.css` 201KB / `index-*.js` 370KB。
- `cargo fmt --check` 通過。
- `cargo check` 通過。
- `cargo clippy -- -D warnings` 通過。
- Tauri WebView 視覺驗收尚未進行：依 `.claude/rules/claude-code-background-shell.md` 規則，AI session 內禁止背景起 `cargo tauri dev`，須由開發者用 `dev-start.command` 開啟桌面 app 確認。

## Technical Notes

- 改寫後 shell 不再依賴 `@mantine/core` 的 `AppShell`/`Indicator`/`NavLink`，rail active 狀態自行用 `useRouterState()` 推導 `currentPath`。
- `surface` 樣式（panel-border + panel-bg + 6px radius）成為內容卡片唯一基底，避免 `Card` 與自製 panel 樣式漂移。
- Rail status badge 使用 9px / 0.08em letter-spacing 維持高密度 metadata 標籤一致性。
- DESIGN.md 與 shared style guide 同時保留：repo-specific 規則只寫 Realestate 差異化（amber evidence flag、emerald rail status），共用部分指回 shared guide。

## Untracked 觀察（非本 session 改動）

`git status` 仍有三筆 untracked，疑似 Project-Manager 工具今天稍早於 `2026-05-18T03:21Z` 自動生成：

- `.project-manager.json`：工具 config。
- `docs/dev-logs/.gitkeep`：空目錄。**注意路徑與 `.claude/rules/general.md` 規定的 `docs/project-process/dev-logs/` 衝突**，留著可能誤導未來人。
- `docs/features/.gitkeep`：空目錄。

建議：開發者決定要把 `.project-manager.json` 加入 `.gitignore` 還是 commit；空殼 `docs/dev-logs/` 與 `docs/features/` 建議刪除或加入 `.gitignore`，避免與 `docs/project-process/` 結構分歧。

## Next Priority

- 視覺驗收：用 `dev-start.command` 開桌面 app 與瀏覽器 5173，檢查 rail active state、寬窄螢幕（<1200px 隱藏右 panel、<840px 隱藏 rail）、Settings 頁 storage diagnostics 排版。
- 建立前端測試 harness（Vitest + Mantine wrapper），允許後續 Settings 與 shell 的 smoke test。
- 評估 production `.app` / `.dmg` bundle：icon 已備齊，下一步是 signing 設定與 release pipeline 草案。
- 開始 Phase 2 前先在 ADR 寫清楚 property/document 資料邊界（哪些走 server HTTP、哪些只存本地 cache）。
