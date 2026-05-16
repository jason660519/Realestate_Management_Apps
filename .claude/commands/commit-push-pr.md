# Commit Push PR

根據目前的 git diff 與 untracked files，執行整合流程（commit → PR → merge/cleanup → handoff）。

## 適用參數

- `--pr` / `--pull-request`：建立 PR
- `--auto-merge`：建立 PR 後自動檢查並嘗試合併
- `--cleanup`：PR 合併後清理分支
- `--handoff`：產出 handoff
- `--full-auto`：等同 `--pr --auto-merge --cleanup --handoff`

## 步驟一：Commit 與 Push

1. 執行 `git status` 與 `git diff`，分析變更。
2. 排除敏感檔案（`.env`、`credentials.json`、`*.key`），不得加入 staging。
3. 逐檔 `git add`（避免 `git add -A`）。
4. 維持「單一主題、單一 commit」原則。
5. 產生 commit message：`<type>: <繁體中文描述>`
   - type: feat / fix / docs / refactor / style / test / chore
6. 執行 commit。
7. push 到目前分支。

## 步驟二：建立 PR（可選）

當含 `--pr` 時執行：

1. 用 `git log` 與 `git diff main...HEAD` 分析 commits。
2. `gh pr create`：
   - Title：簡短描述（< 70 字元）
   - 合併策略：Squash and merge
3. PR Body 格式：

```markdown
## Summary
- <變更摘要>

## Test plan
- [ ] cargo test
- [ ] cargo clippy
- [ ] npm run typecheck

## Merge strategy
- Squash and merge
```

4. 回報 PR URL。

## 步驟三：審核與合併（可選）

當含 `--auto-merge` 時：

1. `gh pr view --json` 讀取狀態。
2. 條件全成立時 merge：非 draft、無 conflict、CI 全綠、無 CHANGES_REQUESTED。
3. `gh pr merge --squash --delete-branch=false`
4. 條件不符合時回報阻塞原因。

## 步驟四：清理分支（可選）

當含 `--cleanup` 時：

1. 確認 PR 已 merge。
2. 禁止刪除 `main`、`master`、`develop`。
3. 刪除遠端 + 本地分支。

## 步驟五：產出 Handoff（可選）

當含 `--handoff` 或 `--full-auto` 時，依 `.claude/commands/handoff.md` 執行。

## 注意事項

- 不要 commit secrets
- 不要 force push
- 不要 amend 既有 commit（除非明確要求）
- 不要刪保護分支
- commit message 描述使用繁體中文

## 特別強調

$ARGUMENTS
