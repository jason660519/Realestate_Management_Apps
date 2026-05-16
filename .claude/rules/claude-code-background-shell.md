# Claude Code Background Shell — 已知漏水點

> 寫新的長壽進程（dev server / watcher / docker compose up / tail -f）之前**必讀**。
> 違反這條規則的後果：磁碟被 `/private/tmp/claude-{UID}/tasks/` 吃掉幾十 GB，且**沒有自動清理機制**。

---

## 問題描述

Claude Code 對 `run_in_background: true` 的子進程會在 OS 層攔截 stdout/stderr，存到：

```
/private/tmp/claude-{UID}/tasks/{task_id}.output
```

**沒有 rotation、沒有 size limit、沒有自動清理**。

---

## 規則

### 禁止

- **禁止**在 Claude Code 對話內用 `run_in_background: true` 起 `cargo tauri dev` / `npm run dev` / `trunk serve` / `vite` / `docker compose up`（不加 `-d`）等長壽進程。
- **禁止**寫「我先在背景開 dev server，等一下回來看 log」這類流程。
- 使用者自己在獨立 Terminal 視窗起 dev server，Claude 只負責讀 log file 或打 health check。

### 允許

- **短任務**（< 30 秒、輸出有限）：可以用 background，但跑完**必須** kill。
- **Docker**：`docker compose up -d`（detached mode）可以，因為 log 走 Docker 自己的 driver。

---

## 定期維護

```bash
# 每週檢查
du -sh /private/tmp/claude-* 2>/dev/null

# 清理（確認沒有 active session）
rm -rf /private/tmp/claude-$(id -u)/tasks/*
```

---

## 歷史教訓

舊專案 `Owner-Property-Management-AI-SPA` 曾因多個 worktree 各自起 `nohup npm run dev`，導致 `/private/tmp/claude-*/` 累積到 79GB。本規則源自該事件。
