# Wrap Up — Session 收尾全自動

Session 結束時的「一鍵收尾」編排器。依序串接：寫日誌 → commit → push → PR → merge → cleanup → handoff。

> **本檔是薄編排器**，實際邏輯在子命令裡。

---

## 觸發詞

- `/wrap-up`
- 「收尾」「session 結束」「一鍵收工」「全自動收尾」「ship it」

## 預設模式

等同 `--full-auto`：
- 產 dev-log
- commit / push / 開 PR
- 自動 merge（條件符合時）
- 刪分支
- 產 handoff

## 可選參數

| 參數 | 行為 |
|:--|:--|
| `--no-report` | 跳過 dev-log |
| `--no-merge` | 只開 PR，不 merge |
| `--no-handoff` | 跳過 handoff |
| `--dry-run` | 印出將執行的步驟，不實際動手 |

---

## 流程（4 步）

### 步驟 1 — 產 dev-log

撰寫 `docs/project-process/dev-logs/dev-{topic}-{YYYY-MM-DD}.md`：
1. 本次完成任務清單
2. 技術困難與解法
3. 下次優先工作

> 若本次 session 完全沒有實質交付，跳過。

### 步驟 2 — Sanity check

```bash
git status --short
git diff --stat
cargo fmt --check
cargo clippy -- -D warnings
```

確認無 secrets、無非預期大改動、Rust checks 通過。異常時中斷並回報。

### 步驟 3 — Commit / Push / PR / Merge / Cleanup

依 `.claude/commands/commit-push-pr.md` 的 `--full-auto` 模式執行。

### 步驟 4 — Handoff

依 `.claude/commands/handoff.md` 執行。

---

## 失敗處理

| 失敗位置 | 行為 |
|:--|:--|
| 步驟 1（dev-log） | 中斷，回報 |
| 步驟 2（sanity） | 中斷，等 user 決定 |
| 步驟 3（git） | 中斷，回報錯誤 |
| 步驟 3（merge 被擋） | 繼續到步驟 4，handoff 記錄阻塞原因 |
| 步驟 4（handoff） | 重試一次；仍失敗則手動產 chat block |

---

## 完成輸出格式

```
✅ Wrap-Up 完成

📄 Dev Log:    docs/project-process/dev-logs/dev-{topic}-{YYYY-MM-DD}.md
🔧 Commit:     {sha} <type>: <描述>
🔀 PR:         {URL}（{merged|open|blocked}）
🧹 Cleanup:    {分支名} 已刪除
📋 Handoff:    docs/project-process/handoffs/handoff-{topic}-{YYYYMMDD}.md

下一步建議：{1-2 句}
```

---

## 注意事項

- 回覆繁體中文，程式碼註解英文
- 不要 commit secrets
- 不要 force push
- 不要刪保護分支

## 特別強調

$ARGUMENTS
