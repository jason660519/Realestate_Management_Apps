根據本次 session 已完成的工作，寫一份**自包含**的接手 prompt 給下一個 AI session 使用。
使用者會開新 session 把你的輸出整份貼進去，新 AI 必須**從零**就能理解並直接動工。

## 觸發條件

凡看到以下任一表達，**都是本 command 的觸發訊號**：

- 直接輸入 `/handoff`
- 「寫 handoff」「handoff prompt」「寫接手 prompt」「接手 prompt」
- 「給下一個 session / AI 的指引 / 交接 / 接手」
- 使用者在 session 尾端要求「收尾」「結束前交代一下」

**絕對禁止**：
- ❌ 在 chat 裡 ad-hoc 手寫 markdown 當成 handoff，卻沒按本流程執行
- ❌ 只產 chat fenced block、略過「B. 同時存成檔案」
- ❌ 用口頭引導取代實際 `Write` 工具落地

## 動筆前必做的現場偵察

1. `git status --short` + `git log --oneline -10` — 確認本次 session 的 commit 與未提交變更
2. 讀本次 session 最後 commit 的核心檔案（看**現況內容**）
3. 讀 `docs/product/prd.md` 相關章節確認與 product direction 一致
4. 掃 `.claude/rules/` 與 `CLAUDE.md`，把非直覺慣例挑出來
5. **驗證你要在 prompt 中提及的每個技術斷言** — grep/Read 驗證後才能寫

## 輸出格式（雙軌：chat + 檔案）

### A. Chat 輸出
用 markdown fenced code block 包住整份 prompt，讓 user 一鍵複製。

### B. 同時存成檔案

存到 **`docs/project-process/handoffs/handoff-{topic}-{YYYYMMDD}.md`**：

```markdown
# Handoff — {主題}

> **產出時間**：YYYY/MM/DD
> **產出者**：Claude {model}（與 {使用者} 對話）
> **接手對象**：下一個 Claude session
> **承接內容**：（一句話）
> **如何使用**：複製下方 fenced code block 整段，貼到新 session 的第一則 prompt

---

（fenced 內容）

---

## 相關文件
（cross-link 到 ADR、PRD、dev-log）
```

### 內含段落（依序）：

1. **身分與慣例** — 繁中、英文註解、Rust clippy 全過、TS strict 禁 any
2. **專案位置** — 絕對路徑
3. **剛完成** — commit hash + 交付物路徑
4. **驗證基線指令**：
   ```
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   npm run typecheck
   ```
5. **下一步任務拆解** — 要建哪些檔案、用什麼 crate/套件、為什麼
6. **延後 / 待辦**
7. **關鍵慣例與雷區** — Tauri allowlist、server config 不 hardcode、evidence-first 原則
8. **驗收門檻** — 完成條件 + commit message 格式
9. **動工前確認指令** — `git status` + `git log --oneline -5`

## 品質要求

- 每個檔案路徑寫完整路徑
- 技術斷言必須附證據（grep 結果、檔案路徑）
- 結尾留一句：動工前先跟我確認任務拆解，避免悶頭寫錯方向

## 完成檢查清單

- [ ] Chat 已輸出 fenced code block
- [ ] 已建立 `docs/project-process/handoffs/handoff-{topic}-{YYYYMMDD}.md`
- [ ] 在 chat 結尾告知使用者兩個位置

## 特別強調

$ARGUMENTS
