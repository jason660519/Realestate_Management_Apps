針對難以復現的 bug 與效能回歸，執行紀律性診斷迴圈。
流程：建立回饋迴圈 → 復現 → 假設 → 儀器化 → 修復 → 回歸測試。

---

## Phase 1 — 建立回饋迴圈（最重要）

有了快速、確定性、可自動執行的 pass/fail 信號，bug 就解決了 90%。

### 本專案可用的回饋迴圈工具（依優先序）

1. **Rust test** — `cargo test` specific module
2. **Tauri command test** — integration test in `tests/`
3. **curl / HTTP script** — 打 server API 並 diff 回應
4. **前端 test** — component test 或 E2E
5. **最小復現腳本** — 隔離問題路徑
6. **Bisect** — `git bisect run <test_script>`
7. **差異比對** — 舊版 vs 新版對同輸入 diff 輸出

迴圈建好後繼續優化它：更快？信號更精確？

**若真的無法建迴圈**：停下來明說。列出試過的方法。請使用者提供環境存取或 log dump。**不能在沒有迴圈的情況下進入 Phase 2。**

---

## Phase 2 — 復現

執行迴圈，確認 bug 出現。

- [ ] 迴圈產生使用者描述的失敗模式
- [ ] 失敗可復現
- [ ] 已記錄精確症狀

---

## Phase 3 — 假設

**先列出 3–5 個排序假設，再測試任何一個。**

每個假設必須可被證偽：
> 「如果 \<X\> 是原因，那麼 \<改動 Y\> 會讓 bug 消失。」

**開始測試前把排序清單給使用者看。**

---

## Phase 4 — 儀器化

每個探針必須對應 Phase 3 的某個預測。一次只改一個變數。

工具優先序：
1. **Debugger / REPL** — 一個 breakpoint 勝過十條 log
2. **精準 log** — 只在區分假設的邊界上加
3. 絕不「全部 log 再 grep」

**每條 debug log 加唯一前綴**，例如 `[DEBUG-a4f2]`。清理時一個 grep 搞定。

---

## Phase 5 — 修復 + 回歸測試

1. 把最小復現轉成失敗測試
2. 看它失敗
3. 施加修復
4. 看它通過
5. 重跑 Phase 1 迴圈

---

## Phase 6 — 清理 + 事後分析

- [ ] 原始復現不再復現
- [ ] 回歸測試通過
- [ ] 所有 `[DEBUG-...]` 儀器已移除
- [ ] commit 訊息說明哪個假設正確

**最後問：什麼能預防這個 bug？**

$ARGUMENTS
