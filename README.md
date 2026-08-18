# Glucose Dashboard

本地優先（local-first）的血糖儀表板：指向一張 Google Sheet，Rust 後端隨需
讀取、計算分析，並以 React UI 呈現。無伺服器、無資料庫——瀏覽器是唯一的
操作介面。Google Sheet 為唯一資料來源，系統不持久化 Sheet 資料。

## 安裝並啟動（Windows，一行指令）

在 PowerShell 執行：

```powershell
irm https://raw.githubusercontent.com/jamesai20139-maker/GlucoseDashboard/main/installer/get.ps1 | iex
```

腳本會下載最新版的單一可執行檔（前端已嵌入）、安裝到
`%LOCALAPPDATA%\GlucoseDashboard`、加入使用者 `PATH`，並直接啟動
（後端會自動開啟瀏覽器）。之後在新開的 PowerShell 輸入 `glucose-dashboard`
即可再次啟動。詳見 [installer/README.md](installer/README.md)。

## 使用者資料填寫參考

儀表板需要一份符合固定欄位格式的 Google Sheet 才能讀取。若你是第一次建立資料表，請參考：

- [docs/填寫說明.md](docs/填寫說明.md) — 詳細說明欄位格式、日期與事件規則、無效資料處理、設定注意事項。
- [docs/sample-sheet.csv](docs/sample-sheet.csv) — 可直接複製貼入 Google Sheet 的範例檔，涵蓋六個內建事件與備註寫法（含 UTF-8 BOM，可用 Excel 預覽）。

## 開發者

開發與測試請見 [CLAUDE.md](CLAUDE.md) 與 [Agents.md](Agents.md)。

```bash
make run     # 建前端 + 套用設定 + 啟動後端（127.0.0.1:3000）
make test    # 後端 cargo test + 前端型別檢查 / build
```

發布新版本：打 tag 觸發 CI 編譯單一 exe 並建立 Release（見
[.github/workflows/release.yml](.github/workflows/release.yml)）。

```bash
git tag v0.2.0 && git push origin v0.2.0
```
（首次發布前先把 repo 設為 public，並確認帳號未受限制，否則 raw 安裝連結與 Actions 皆無法對外生效。）