# Rust + Tauri 開發慣例

---

## 專案結構（scaffold 後）

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root (re-exports)
│   ├── commands/            # Tauri invoke handlers
│   │   ├── mod.rs
│   │   ├── health.rs        # Server health check commands
│   │   ├── property.rs      # Property CRUD commands
│   │   └── document.rs      # Document intake commands
│   ├── services/            # Business logic (no Tauri dependency)
│   │   ├── mod.rs
│   │   ├── server_client.rs # HTTP client to internal server
│   │   └── config.rs        # App configuration
│   ├── models/              # Domain types
│   │   ├── mod.rs
│   │   ├── property.rs
│   │   ├── document.rs
│   │   └── evidence.rs      # AI stage trace types
│   └── errors.rs            # Error types (thiserror)
└── tests/                   # Integration tests
```

---

## Tauri Command Surface 規則

### 什麼走 Rust invoke（Tauri command）
- App 設定讀寫（本地 config）
- Server connection 管理（health check, reconnect）
- 檔案選取/匯入（需 filesystem 權限）
- Local cache / queue metadata 操作
- Plugin registry 狀態

### 什麼走 HTTP 到 Server
- Property CRUD（正式資料）
- Document processing（OCR、AI）
- GIS 查詢
- Search / indexing
- AI model routing

### WebView 不可直接做
- 任意 filesystem access
- Shell execution
- Secret 讀取
- 直接打外部 AI provider API（必須經過 server 或 Rust layer）

---

## Error Handling

```rust
// Domain errors: 用 thiserror
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Server unreachable: {0}")]
    ServerUnreachable(String),
    #[error("Document processing failed at stage {stage}: {reason}")]
    ProcessingFailed { stage: String, reason: String },
    // ...
}

// Binary / glue code: 用 anyhow（僅限 main.rs 和 commands/）
// Services 層必須用 typed error
```

---

## Tauri Config 安全原則

```json
// tauri.conf.json allowlist 最小權限
{
  "tauri": {
    "allowlist": {
      "all": false,
      "dialog": { "open": true, "save": true },
      "fs": { "scope": ["$APPDATA/*", "$DOWNLOAD/*"] },
      "http": { "scope": ["http://192.168.1.*:*/*"] },
      "shell": { "all": false }
    }
  }
}
```

- `fs.scope` 只開 app 資料夾和使用者選取的路徑
- `http.scope` 只開內網 server 範圍
- `shell` 預設全關

---

## 測試慣例

### Unit Test
```rust
// 放在同 module 內
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_validation() { ... }
}
```

### Integration Test
```rust
// tests/ 目錄，測 command 完整流程
#[tokio::test]
async fn test_server_health_check() { ... }
```

### 執行
```bash
cargo test                     # All tests
cargo test --lib               # Unit only
cargo test --test server_health # Specific integration test
```

---

## 常見陷阱

1. **Tauri state 管理**：用 `tauri::State<T>` 注入 shared state，不要用 `lazy_static` 或 global mut
2. **Async runtime**：Tauri commands 可以是 async，底層用 tokio。但 WebView 回調在 main thread
3. **Serde 命名**：Tauri command 參數自動 camelCase 轉換（JS → Rust）。若自定義用 `#[serde(rename_all = "camelCase")]`
4. **Window event**：長時間操作用 `window.emit()` 推進度到前端，不要讓 invoke 長時間 hang

---

## Config 管理

```rust
// Server address 從 config 讀，絕不 hardcode
// Config 路徑：$APPDATA/realestate-management/config.toml
// 允許 UI 修改、重啟生效

[server]
base_url = "http://192.168.1.6:8080"
health_check_interval_sec = 30
timeout_sec = 10

[plugins]
saydo_enabled = false
project_manager_enabled = false
```

---

## 依賴版本鎖

scaffold 後在 `Cargo.toml` 中用精確版本或 `~` 鎖定關鍵依賴：

```toml
[dependencies]
tauri = { version = "2", features = ["..."] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
thiserror = "2"
```

更新 major 版本需 ADR 記錄。
