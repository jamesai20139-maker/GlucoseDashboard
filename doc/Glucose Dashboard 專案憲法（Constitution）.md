# Glucose Dashboard 專案憲法（Constitution）

Version 1.0

---

# 一、專案願景（Project Vision）

Glucose Dashboard 是一套以本機（Local）執行為核心的血糖分析儀表板。

本專案希望讓一般使用者，不需要架設 Server、不需要安裝資料庫，也不需要具備程式能力，即可透過 Google Sheet 完成血糖資料分析與視覺化。

本專案應始終追求：

* 零架設（Zero Setup）
* 零維護（Zero Maintenance）
* 一鍵安裝（One Command Install）
* 一鍵啟動（One Command Start）
* 自動更新（Auto Update）
* 高效能（High Performance）
* 易於維護（Maintainable）

---

# 二、產品核心原則（Core Product Principles）

## 2.1 Local First

所有核心功能皆應於本機執行。

除 Google Sheets API 外，不應依賴任何雲端服務才能完成主要功能。

---

## 2.2 Google Sheet 為唯一資料來源

Google Sheet 為系統唯一正式資料來源（Single Source of Truth）。

系統不得建立自己的資料庫。

不得將 Google Sheet 的資料同步至永久性資料庫。

所有分析結果皆應由 Google Sheet 即時計算而來。

---

## 2.3 專注分析，而非資料管理

本產品定位為：

* 血糖分析工具
* 血糖視覺化工具
* 健康趨勢觀察工具

本產品不是醫療系統。

本產品不負責管理病歷。

Google Sheet 持續作為資料管理工具。

Dashboard 專注於分析與視覺化。

---

# 三、架構原則（Architecture Principles）

系統架構固定為：

```text
Google Sheet
        │
Google Sheets API
        │
Rust Backend
        │
REST API
        │
React Dashboard
        │
Browser
```

不得隨意改變此架構。

Frontend 與 Backend 必須保持低耦合。

Frontend 僅透過 REST API 與 Backend 溝通。

不得直接存取 Backend 內部資料。

所有商業邏輯皆應放置於 Backend。

Frontend 僅負責畫面呈現與使用者互動。

---

# 四、技術原則（Technology Principles）

Backend 技術：

* Rust
* Cargo
* Axum
* Tokio

Frontend 技術：

* React
* TypeScript
* Vite

執行環境：

* 系統預設 Browser

不得導入：

* Electron
* Tauri

除非經過重大架構決策（ADR）同意。

---

# 五、資料原則（Data Principles）

Google Sheet 為唯一正式資料來源。

Google Sheet Header 視為公開介面（Public Contract）。

不得任意修改：

* Header 名稱
* Header 順序
* Header 意義

所有 Google Sheet 資料，皆應先轉換為 Domain Model：

```text
BloodGlucoseRecord
```

所有分析邏輯皆應操作 Domain Model。

不得直接操作 Google Sheet Row。

---

# 六、CLI 原則（CLI Principles）

CLI 應具備成熟工具的使用體驗。

設計原則：

* 指令簡潔
* 命名一致
* 易於記憶
* 易於診斷
* 易於擴充

一般使用者應能透過單一指令完成：

* 安裝
* 啟動
* 更新
* 設定
* 系統檢查

---

# 七、安裝與更新原則

安裝流程應儘可能簡化。

目標：

* 一行指令完成安裝
* 一個指令完成啟動

更新時：

* 不得遺失設定
* 不得要求重新設定 Google Sheet
* 優先保留使用者環境

---

# 八、Dashboard 設計原則

Dashboard 採用 Information First 設計理念。

所有重要資訊應於第一個畫面即可看到。

不得要求使用者大量切換頁面。

Dashboard 應：

* 高可讀性
* 專業醫療風格
* 最少操作步驟
* 快速載入
* 一致性的元件設計

任何篩選條件改變後：

* Summary Cards
* Trend Chart
* Detail Table

皆應同步更新。

---

# 九、效能原則（Performance Principles）

效能為產品核心價值之一。

目標：

* Dashboard 3 秒內完成啟動
* Rust Backend 記憶體使用量低於 100 MB（正常使用情境）
* 保持輕量化安裝
* 使用者操作應即時回應

任何效能退化皆視為缺陷。

---

# 十、程式設計原則（Engineering Principles）

所有功能皆應：

* 模組化
* 可維護
* 可測試
* 可重用

修改既有功能時：

優先修改現有模組。

避免重新設計整個系統。

除非經過重大架構討論，不應進行大規模重構。

---

# 十一、AI Agent 開發原則

所有 AI Agent 在開發時皆必須遵守：

1. 保持既有架構。
2. 維持 Frontend 與 Backend 低耦合。
3. Google Sheet 永遠為唯一資料來源。
4. 優先確保安裝、更新與啟動流程穩定。
5. 每次修改應保持最小變更範圍。
6. 每項功能皆須可獨立驗證。
7. 每次修改皆不得破壞既有功能。
8. 新功能優先整合既有架構，而非新增重複功能。

---

# 十二、未來擴充原則

未來新增功能應建立於既有架構之上，而非推翻原有設計。

包含但不限於：

* AI 血糖分析
* 趨勢預測
* PDF 報表
* 多個 Google Sheet
* 多位使用者
* Plugin 機制
* Apple Health / Google Fit 整合

所有新功能皆不得違反本憲法。

---

# 十三、決策優先順序（Decision Priority）

當不同設計方向發生衝突時，應依照以下優先順序決策：

1. 保持產品簡單易用
2. 保護資料完整性
3. 維持既有架構
4. 保持程式可維護性
5. 提升使用者體驗
6. 確保系統效能
7. 保留未來擴充能力

所有開發決策皆應符合上述原則。

---

# 十四、憲法優先權

本文件為 Glucose Dashboard 專案最高開發原則。

所有 PRD、Specification、Plan、Tasks 與 Implementation 均不得違反本憲法。

若需求與本憲法發生衝突，應優先檢討需求或提出 Architecture Decision Record（ADR），而非直接修改本憲法。
