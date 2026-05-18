# ADR-009: Production Bundle, Code Signing, and Auto-Update

Status: Proposed
Date: 2026-05-18
Owner: Project lead
Related: ADR-002, ADR-003, `docs/product/prd.md` §6.1, `src-tauri/tauri.conf.json`

## Context

Phase 1 已可在 dev 模式跑（`cargo tauri dev` / `dev-start.command`），app icon 已備齊，config 持久化已落地。下一個交付里程碑是給使用者可安裝的 `.app` / `.dmg`。要決定：

1. v0 → v1 的 bundle target 範圍（先做什麼平台？）
2. macOS code signing 策略（v0 unsigned 還是直接 sign？notarization 何時做？）
3. Auto-update 機制（v0 不做？接 Tauri updater plugin？）
4. DMG packaging 細節（layout、background、license）
5. Linux / Windows 何時納入

目前 `tauri.conf.json` 設 `bundle.targets: "all"`，未設 `bundle.macOS.signingIdentity`、`updater` 或 entitlements。Cargo.toml 沒列 `tauri-plugin-updater`。

## Decision

### 1. v0 Bundle Target：macOS only（.app + .dmg）

理由：使用者只在 macOS 上操作（PRD §6.1 桌面定位 + 開發機 = macOS），Linux / Windows 沒有立即需求。

`tauri.conf.json` 改為：

```json
{
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    ...
  }
}
```

`"all"` 在 macOS host 實際只會出 macOS targets，但顯式列出避免後續誤解。

Linux / Windows targets 待 Phase 3 跨平台需求出現時新增（屆時可能需要 CI host 或 GitHub Actions runner）。

### 2. macOS Code Signing：v0 unsigned，v1 上 Developer ID

v0 不簽（unsigned `.app`）。使用者第一次開啟需 right-click → Open 略過 Gatekeeper。在 README 與 release note 寫清楚步驟。

v1（首次外部交付前）：

- 取得 Apple Developer ID Application 憑證
- 設 `tauri.conf.json`：
  ```json
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: <Name> (<TeamID>)",
      "hardenedRuntime": true,
      "entitlements": "entitlements.plist"
    }
  }
  ```
- entitlements 預設保守（不開 `com.apple.security.cs.allow-unsigned-executable-memory` 等）；keychain access entitlement 看 ADR-004 secret 實作需要再決定

v1.5：Notarization（需 Apple ID + app-specific password）：

- Tauri 2 內建 `notarize` 流程，用 `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` 環境變數
- 加進 release script，不入 git

### 3. Auto-Update：v0 不做，v1.5 評估 Tauri updater plugin

v0 假設使用者重新下載 `.dmg` 安裝，不掛 auto-update。理由：v0 內網 desktop tool 變更頻率不高、updater 需要簽章 + endpoint 託管，先延後。

v1.5 評估 Tauri updater plugin：

- 加 `tauri-plugin-updater` 到 Cargo.toml
- 設 `updater.endpoints`（建議內網 server `https://192.168.1.6/updates/realestate/{{target}}/{{current_version}}` 或同 server 上某靜態路徑）
- Sign 用 minisign / ed25519（updater 內建），key 不入 git
- UI：Settings 頁加「檢查更新」按鈕，發現新版時人類確認後下載（不自動安裝）

不採方案：
- Sparkle（macOS 專屬，與 Tauri 整合需另接）
- GitHub Releases 為 updater endpoint（v0 不公開到外網）

### 4. DMG Packaging：v0 預設 layout，v1 客製 background

v0 用 Tauri DMG 預設外觀（app + Applications symlink）。不指定 background 圖。

v1 加：

```json
"bundle": {
  "macOS": {
    "license": "../LICENSE",
    "frameworks": []
  },
  "dmg": {
    "background": "assets/dmg-background.png",
    "appPosition": { "x": 180, "y": 220 },
    "applicationFolderPosition": { "x": 480, "y": 220 },
    "windowSize": { "width": 660, "height": 400 }
  }
}
```

DMG background 由 `app-icon.svg` 同系設計（emerald / amber），由 Realestate 視覺指南產出（DESIGN.md / shared-ai-desktop-style.md）。

### 5. Linux / Windows：Phase 3 才規劃

不在 v0/v1 範圍。當需求出現時新增 ADR，內容涵蓋：
- Linux：`.deb` + `.AppImage` + keyring fallback（headless / SSH 環境，呼應 ADR-004 Open Q2）
- Windows：`.msi` + Credential Manager 行為差異
- CI：是否需要 GitHub Actions 或自架 runner

## Bundle Identifier 與 Version 管理

- `identifier`：`com.realestate-management.desktop`（已定，不變；與 ADR-004 keychain service name 一致）
- `version`：跟著 `package.json` + `Cargo.toml`，手動 bump。語意化版本 `0.x.y` 直到 v1 release，之後 `1.x.y`
- `productName`：`Realestate Management Apps`（已定）

Release 流程（v0）：

1. Bump version in `package.json` + `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json`
2. `npm run typecheck && npm run test && npm run build`
3. `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
4. `npm run tauri:build`
5. 產出在 `src-tauri/target/release/bundle/{macos,dmg}/`
6. 手動測試：double-click `.dmg` → drag to Applications → 開啟確認 config 路徑、health check、icon 正確
7. 將 `.dmg` 上傳到內網 share（路徑由 `internal-server-plan.md` 決定）

## Consequences

### 正面

- v0 馬上能交付給內部使用者試用，不卡簽章
- Signing / notarization / updater 的決策路徑明確，當需要時知道做什麼
- Bundle 設定集中在 `tauri.conf.json`，不散落 ad-hoc script

### 負面 / 成本

- Unsigned `.app` 首次開啟體驗差（Gatekeeper 警告），需 README 教學
- Notarization 與 Developer ID 帳號有費用（Apple Developer Program $99/年）
- Updater 延後 → v1 之前更新都要手動重 install

### 風險

- macOS Keychain access prompt：未簽章 app 每次讀 keychain 都會彈窗（ADR-004 Open Q3）。Mitigation：v0 先不把 secret 寫 keychain，或接受 prompt；v1 簽章解
- DMG layout 不客製 → 外觀像未完工 app。風險低（內部使用者），v1 補

## Open Questions

1. **何時取得 Apple Developer ID？** 建議：v0 → v1 過渡點（首次外部交付 / beta 使用者），需專案 owner 申請帳號
2. **Updater endpoint host 用 server 上哪個 service？** 建議：reverse-proxy 加 `/releases/realestate/` 路徑，指向 server filesystem 或 MinIO。等 `internal-server-plan.md` 補
3. **是否做 universal binary（Intel + Apple Silicon）？** 建議：v0 只出 Apple Silicon（開發機與目標機都是 ARM）；v1.5 評估 universal 看是否仍有 Intel 使用者
4. **License 寫什麼？** 私有專案內部使用，先放 `Proprietary / Internal Use Only`；若未來開源再換 MIT / Apache-2.0

## Verification

v0 acceptance：

- [ ] `npm run tauri:build` 成功產出 `.app` + `.dmg`
- [ ] `.dmg` 可掛載、拖到 Applications、可開啟（手動 right-click → Open）
- [ ] App 啟動後 config.toml 寫到 `~/Library/Application Support/com.realestate-management.desktop/`（呼應 ADR-004）
- [ ] App icon 在 Dock / Finder / about dialog 正確顯示
- [ ] Settings 頁 storage diagnostics 顯示正確路徑與檔案狀態

v1 acceptance（追加）：

- [ ] `.app` 通過 `codesign --verify --deep --strict` 檢查
- [ ] `.dmg` 通過 `spctl --assess --type open --context context:primary-signature` 檢查
- [ ] Notarization ticket attached（`stapler validate`）
- [ ] Updater 從內網 endpoint 找到 latest，UI 顯示更新提示，人類確認後下載完成
