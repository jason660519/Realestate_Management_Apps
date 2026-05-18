# Dev Log: Write Path Desktop Skeleton + Component Test 第二輪

Date: 2026-05-18

第五場（同日累計第七份 dev log）。把 ADR-010 的 write path 在 desktop
端先立骨架（不等 axum service 上線），同時補純元件 component test
作為後續 React UI 測試的範本。

## Completed

### A. `ServerClient::post_json` generic 寫入入口

- 新增 `post_json::<TIn, TOut>(path, body)` 方法：
  - `TIn: Serialize + ?Sized`、`TOut: DeserializeOwned`
  - reqwest 用 `.json(body)` 自動 set `Content-Type: application/json`
  - 非 2xx 包成 `AppError::ServerUnreachable`，**含 body snippet**（trimmed），讓 UI 顯示 server 端 validation 訊息（呼應 ADR-010 §6 Error contract）
  - JSON parse 失敗也包同型 error 但訊息含 path
- 加 2 個 unit test：
  - 成功路徑：mock server 收 POST body 並回 200 + JSON
  - 4xx 路徑：mock server 回 400 + `{"kind":"validation","message":"display_name is required"}`，確認 error message 同時含 status code 與 body snippet

### B. `save_property` skeleton

- `models.rs` 新增：
  - `SavePropertyPayload`（minimal 版）：`id (Option<String>)`、`display_name`、`kind`、`address_raw`
  - `PropertyMutationResponse`：`id`、`updated_at`
  - 兩者皆帶 `#[serde(rename_all = "camelCase")]` 與 Tauri 既有 typed surface 對齊（**不**是 PostgREST direct surface，所以走 camelCase）
- `services/property_service::save_property(config, payload) -> Result<PropertyMutationResponse, AppError>`：
  - **不**走 cache：save 必須打到 canonical server 或明確失敗（呼應 boundary doc 不變式「canonical = server」）
  - 本機先驗 `display_name` 非空，省一趟網路（呼應「validate at system boundaries」）
  - 空 base URL 直接回 `InvalidInput`，不走 cache fallback
- `commands/property::save_property` Tauri command（薄殼）
- `lib.rs` 註冊新 invoke handler
- 加 4 個 unit test：成功、空 display_name validation（不走網路）、空 base URL 拒絕、server 端 400 validation 透到 error

明確標註為 **skeleton**：完整 evidence-backed payload（含 `EvidenceValue<T>`、actor、reason）會在 axum service 契約敲定後擴展。

### C. 純元件 component test（第二輪）

- `src/components/EmptyOperationalState.test.tsx`：title / detail 渲染、heading level 4
- `src/components/PageHeader.test.tsx`：title heading level 2、eyebrow 文字、children slot 出現 / 不出現

故意挑無 hook 依賴的純元件作為「最低門檻範本」，未來凡接到 hook（`useAppData`、`useQuery`）才需要 wrapper + mock。

## Verification

- `cargo fmt --check`：通過。
- `cargo clippy --all-targets -- -D warnings`：通過。
- `cargo test`：**23 / 23**（既有 17 + post_json 2 + save_property 4）。
- `npm run typecheck`：通過。
- `npm run test`：**25 / 25**（既有 20 + EmptyOperationalState 2 + PageHeader 3）。
- 未重 build `tauri:build`：write path skeleton 不改既有 read path 行為，且 UI 還沒接 save 操作，重 build 沒有實際視覺差異。

## Technical Notes

- `post_json` 把 body snippet 加進 error message 不寫死最大長度：v0 server 端 error body 都是短 JSON（`{kind, message}`），實作上不會超過合理長度；若日後遇到極長 body，再加 truncation
- `save_property` 採 fail-loud：server unconfigured → `InvalidInput`，不模仿 read path 用 cache fallback。理由：write 操作的 source of truth 永遠在 canonical server，本機無法代替接受 confirm-grade evidence（呼應 evidence-first §「Confirmed = canonical」與 boundary doc 不變式 §2）
- Tauri command 自動 camelCase 轉換是因為 struct 上的 `#[serde(rename_all = "camelCase")]`：`SavePropertyPayload` deserialize（JS → Rust，camelCase → snake_case）與 `PropertyMutationResponse` serialize（Rust → JS）都吃這條規則
- Test 用 `127.0.0.1:1` 當 unreachable sentinel：在 macOS 上 connect 會立即收到 RST，不需等 timeout，加速 test 跑時。**但**本次 save_property 的 validation 短路測試（empty display_name / blank URL）根本沒網路 I/O，sentinel 不會真的被連
- `EmptyOperationalState` 與 `PageHeader` 兩個元件沒有 props 變動觸發 re-render 的複雜性，純元件測試比 mount + assert text 還要簡單；但加上 heading level 斷言（H4 / H2）能鎖住與 design system DESIGN.md §Typography 的對應，避免 future regression

## Open Questions

1. **`SavePropertyPayload` 擴展時機**：等 axum service 契約。屆時要加 `EvidenceValue<T>` for 高風險欄位、`actor_id`、`reason`、`expected_version`（用 conflict 解 409）
2. **是否暴露 invoke 出 frontend**：本輪只動 backend，frontend `api/tauri.ts` 尚未加 `savePropertySummary()`。等 UI design 拍板（property 編輯介面長什麼樣）再做
3. **Validation 落點分工**：v0 部分前置在 Rust（display_name 非空），主要邏輯在 axum；要避免兩端 drift。考慮共用 typed crate `realestate-shared`（ADR-010 §Open Q2 已記）

## Next Priority

- Frontend 寫入 UI（PropertyForm）：等 ADR-006/010 對 evidence-backed 欄位編輯介面對齊
- Settings / ShellLayout component test：需要更複雜的 hook mock（useAppData、useQuery、router context），值得另行抽出 `src/test/mockAppData.ts` helper
- 啟動 axum service repo（外部行動，需 jason 決策）
- Cache LRU / 容量上限：ADR-004 §3 規劃但 SQLite 端尚未實作；先觀察實際使用大小
