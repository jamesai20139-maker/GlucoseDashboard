# claude_V2.md

本檔是給 Claude Code 使用的獨立精簡指引，**不依賴任何外部 spec 文件**即可運作。內容涵蓋專案本質、工作規範、建置指令與架構地圖。

## 專案概述

一個 **local-first** 的血糖儀表板，是一個血糖分析工具, 血糖視覺化工具,健康趨勢觀察工具,專注於分析與視覺化,給非開發者的 Windows 使用者使用。
沒有伺服器、沒有資料庫——使用者把 app 指向一份 Google Sheet，Rust 後端依需求讀取、計算分析、回應 React UI。瀏覽器是唯一的 UI 介面。
核心憲法原則：**Google Sheet 是唯一資料來源**；系統絕不自行建立資料庫，也不得在臨時分析之外持久化 Sheet 資料。
UI 與所有面向使用者的訊息皆為 **繁體中文** (zh-TW)。技術指令名稱與欄位代碼保持英文。新增面向使用者的字串時請遵循此慣例。

## Project Guidelines & AI Execution Protocol

- **一次只做一個任務**，完成後暫停等待審查，不要自動接續下一個任務。
- 每個任務都需有單元測試、執行完整測試、且零警告。
- 不得修改規格或計畫文件（如 `spec.md`、`plan.md`）——若有需求差異，改以提出 issue 反映。
- 每個任務完成後：更新受影響的文件、提出**繁體中文**的 git commit message，並**不要自動推送**。
- 完成目前任務後停下來等待審查。
- 如果發現任務有矛盾或不清楚的地方, 請提出方案或疑問與開發者討論
- 修改既有功能時：優先修改現有模組,避免重新設計整個系統,除非經過重大架構討論,不應進行大規模重構。
- 維持 Frontend 與 Backend 低耦合。
- Google Sheet 永遠為唯一資料來源。
- 每次修改應保持最小變更範圍。
- 每項功能皆須可獨立驗證。
- 每次修改皆不得破壞既有功能。
- 新功能優先整合既有架構，而非新增重複功能。
## 核心工作流程 (Core Workflow)
遇到複雜需求、新功能開發或重構時，強制執行「規劃 - 拆解 - 分步執行」三階段：

### Phase 1: Plan (規劃)
- 分析用戶需求、專案現有架構與潛在風險。
- 輸出架構設計構想，明確指出需要修改/新增的模組。
- **此階段嚴禁直接寫入或修改任何產品程式碼。**

### Phase 2: Decompose (拆解)
- 將計畫拆解為獨立、單一職責、可驗證的微型任務 (Micro-tasks)。
- 將任務清單寫入 `.claude/tasks.md`（採用 `- [ ]` 語法）。
- 每個任務必須包含明確的 **驗證標準 (Definition of Done)**。

### Phase 3: Execute (單步執行)
- 每次只執行 `.claude/tasks.md` 中**下一個未完成**的任務。
- 完成後自動執行相關測試或驗證指令。
- 驗證通過後，將 `.claude/tasks.md` 的該項目更新為 `- [x]` 並回報進度。

---

## 專案指令參考 (Project Commands)
- **建置 (Build):** `npm run build`
- **測試 (Test):** `npm run test`
- **單一測試 (Single Test):** `npm run test -- <file_path>`
- **Lint 檢查:** `npm run lint`

---

## 程式碼規範與品質標準 (Coding Standards)
- **錯誤處理:** 所有非同步操作與 API 呼叫必須有明確的 Try-Catch 或錯誤處置機制。
- **測試優先:** 修改或新增功能時，必須確保現有測試覆蓋率不下降，並填寫對應測試。
- **註解風格:** 複雜邏輯需註解「Why」而非「What」。
   


## 架構

### 技術架構

```
Google Sheet
        │
        ▼
Google Sheets API
        │
        ▼
Rust Backend
(Data Fetch + Analysis Engine)
        │
 REST API (localhost)
        │
        ▼
React Dashboard
        │
        ▼
Browser
```

---

### 技術選型

- Backend

* Rust
* Cargo
* Axum (REST API)
* Tokio

主要負責：

* Google Sheets API
* 資料分析
* Cache
* REST API
* 自動更新

---

- Frontend

* React
* TypeScript
* Vite
* Chart Library (待選)

主要負責：

* Dashboard
* 圖表
* 統計資料
* 操作介面

---

- Browser

預設使用系統瀏覽器。

不需要 Electron。

不需要 Tauri。

---
```
## 開發者上傳Github流程
- 開發者會把專案push到github上, 並透過git tag 觸發workflow->
                 →  CI 編譯產物  →  上傳成 Release 資產
                 →  安裝腳本抓 Release 資產  		 
## 使用者安裝
	透過github上的read me, 取得一鍵安裝的powerShell指令如"irm https://raw.githubusercontent.com/jamesa-maker/GlucoseDashboard/main/installer/get.ps1 | iex"
    ,讓使用者使用powershell 就可以安裝使用最新版本			 
				 
```

## 啟動流程

使用者：透過localhost或是make run啟動,並使用瀏覽器打開

系統：

```
讀取設定(設定google sheet)
↓
檢查 Google 認證
↓
啟動 Rust API
↓
啟動 Dashboard
↓
開啟瀏覽器, 輸入http://localhost:3000,即可使用
```
## 效能原則（Performance Principles）

效能為產品核心價值之一。

目標：

* Dashboard 3 秒內完成啟動
* Rust Backend 記憶體使用量低於 100 MB（正常使用情境）
* 保持輕量化安裝
* 使用者操作應即時回應

任何效能退化皆視為缺陷。

---

### 程式設計原則（Engineering Principles）

所有功能皆應：

* 模組化
* 可維護
* 可測試
* 可重用



### 無效列處理（關鍵契約）
缺少必填欄位、無法解析日期、血糖超出範圍（20–600）、或未知事件的列會**從統計排除**，但以 `DataQualityIssue` 附列號回報。無效列之間的有效列仍可用。app 絕不改寫使用者的 Sheet。

## 關鍵檔案與常數
- Sheet 標頭（順序不可變）,順序為：`血糖量測日期時間`、`事件`、`量測血糖值(mg/dl)`、`備註1`、`備註2`。
- sheet 欄位格式, 血糖量測日期時間格式如:2026/8/6 下午 9:58:00,顯示在儀表板上要為:2026/8/6 21:58
- 事件名稱（zh-TW，精確）：`空腹血糖`、`午餐前`、`午餐後`、`晚餐前`、`晚餐後`、`睡前`
- eAG/HbA1c 公式集中於 `analysis/summary.rs`（刻意標示為估算值；若臨床認可公式變更，於此一處修改）。

## 備註
- 根目錄的 `cli.sh` 是為 Claude Code 自身設定替代 model/proxy 別名用的——**不是**應用程式的一部分；產品開發時忽略它。
- 安裝目標：Windows 上的 `%LOCALAPPDATA%\GlucoseDashboard`（見 `installer/README.md`）；更新時不得覆寫 config 檔。
- `make test` 執行後端 `cargo test` 加上 frontend **build**（型別檢查）。後端單元測試在 `backend/tests/unit/`；contract/integration harness 目錄在 `backend/tests/contract/` 與 `backend/tests/integration/`。