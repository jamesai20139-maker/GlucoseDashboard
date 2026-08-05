# 血糖監控儀表板 (Glucose Dashboard)

**Product Requirement Document (RPD)**
Version: 0.1 (Draft)

---

# 1. 專案目標

建立一套由本機(Local)執行的血糖監控儀表板，讓使用者可以透過 Google Sheet 作為資料來源，即時分析並視覺化血糖資訊。

本專案希望做到：

* 不需要自行架設 Server
* 不需要安裝資料庫
* 不需要懂程式
* 一行指令即可安裝
* 一個指令即可啟動
* 自動更新
* 使用瀏覽器作為 Dashboard UI

整個系統以 **Rust + React + Browser** 為核心架構。

---

# 2. 產品理念

使用Google Sheet 為分析的資料來源,不建立自己的資料庫，而是：

Google Sheet
↓

Rust Analysis Engine

↓

Local Dashboard

讓使用者可以繼續利用 Google Sheet 管理資料，而 Dashboard 則專注於分析與視覺化。

---

# 3. 產品定位

本產品定位為：

> 一套零架設、零維護、可一鍵安裝的本機血糖分析工具。

它不是醫療系統。

它是一套：

* 血糖資料分析工具
* 血糖視覺化工具
* 健康趨勢觀察工具

---

# 4. 技術架構

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

# 5. 技術選型

## Backend

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

## Frontend

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

## Browser

預設使用系統瀏覽器。

不需要 Electron。

不需要 Tauri。

---

# 6. 安裝流程

## 初次安裝

使用者只需要：

```powershell
irm https://example.com/install.ps1 | iex
```

安裝程式將：

1. 偵測 Windows
2. 下載最新版
3. 解壓縮
4. 安裝至使用者目錄
5. 建立 PATH
6. 完成安裝

完成後即可使用：

```
glucose-dashboard
```

---

# 7. 啟動流程

使用者：

```
glucose-dashboard
```

系統：

```
讀取設定(設定google sheet)

↓

檢查版本

↓

檢查 Google 認證

↓

啟動 Rust API

↓

啟動 Dashboard

↓

自動開啟瀏覽器
```

例如：

```
http://localhost:3000
```

若瀏覽器已開啟，則直接導向 Dashboard。

---

# 8. 更新流程

使用者：

```
glucose-dashboard update
```

系統：

```
檢查 GitHub Release

↓

比較版本

↓

下載最新版

↓

替換執行檔

↓

保留設定

↓

完成更新
```

未來可支援：

```
glucose-dashboard
```

啟動時自動檢查更新。

---

# 9. CLI 指令

## 啟動

```
glucose-dashboard
```

或

```
glucose-dashboard start
```

---

## 更新

```
glucose-dashboard update
```

---

## 設定

```
glucose-dashboard config
```

---

## 檢查系統

```
glucose-dashboard doctor
```

例如：

```
✓ Google Login

✓ Google Sheet

✓ Internet

✓ Config

✓ Cache

✓ Dashboard
```

---

## 查看版本

```
glucose-dashboard version
```

---

# 10. 使用者故事 (User Stories)

## Story 1

身為一位糖尿病患者，

我希望不用安裝資料庫，

就能立即看到血糖分析。

---

## Story 2

身為第一次使用者，

我希望只需要一行安裝指令，

即可完成安裝。

---

## Story 3

身為一般使用者，

我希望不用了解 Rust、

React、

Node.js，

就能使用本產品。

---

## Story 4

身為使用者，

我希望更新軟體時，

不用重新設定 Google Sheet。

---

## Story 5

身為使用者，

我希望每次執行

```
glucose-dashboard
```

都能自動打開 Dashboard。

---

## Story 6

身為使用者，

我希望 Dashboard 可以自動重新整理，

不用一直重新整理網頁。

---

# 11. 使用者情境 (User Journey)

## 初次使用

```
看到 GitHub

↓

複製安裝指令

↓

貼到 PowerShell

↓

完成安裝

↓

輸入

glucose-dashboard

↓

第一次設定 Google Sheet

↓

開始使用
```

---

## 日常使用

```
開 PowerShell

↓

glucose-dashboard

↓

瀏覽器自動開啟

↓

查看今天血糖
```

---

## 更新

```
收到新版通知

↓

glucose-dashboard update

↓

完成
```

---
# 12.資料來源以及內容
## 資料來源為google sheet
1.欄位名稱依序為[血糖量測日期時間],[量測節點],[量測血糖值],[備註1],[備註2]

# 13. UI Design Specification

本系統採用 Dashboard 介面，提供血糖資料的快速分析與視覺化。

設計目標：

- 一眼掌握血糖健康狀態
- 減少操作步驟
- 重要資訊優先呈現
- 支援日、週、月、季分析
- 所有分析皆依據目前選定時間區間自動更新
- 採 Medical Dashboard 設計風格
- 支援 Light Mode（第一階段）
- Dark Mode（第二階段）

---

# 2. 整體版面配置

```
+--------------------------------------------------------------+
| Header                                                       |
+-----------+--------------------------------------------------+
|           | Summary Cards                                    |
|           +--------------------------------------------------+
|           | Blood Glucose Trend Chart                        |
| Sidebar   +--------------------------------------------------+
|           | Blood Glucose Record Table                       |
|           |                                                  |
+-----------+--------------------------------------------------+
```

版面分為：

- Header
- 左側控制區（Sidebar）
- Summary Cards
- Trend Chart
- Detail Table

---

# 3. Header

Header 固定於畫面最上方。

包含：

- 系統名稱
- 最後更新時間
- 手動更新按鈕
- Theme（預留）
- Settings（預留）

Header 不放任何分析資訊。

---

# 4. 左側控制區（Sidebar）

Sidebar 為整個 Dashboard 的控制中心。

所有分析皆依據 Sidebar 的條件重新計算。

## 4.1 時間區間

提供快速切換：

- 日
- 週
- 月
- 季

提供：

- 自訂日期區間（Date Range Picker）

例如：

2026/07/01 ~ 2026/07/31

切換後：

所有 Summary Cards

Trend Chart

Detail Table

全部立即更新。

---

## 4.2 血糖分類篩選

提供：

- 全部
- 空腹血糖
- 午餐前
- 午餐後
- 晚餐前
- 晚餐後
- 睡前

切換後：

Trend Chart

Detail Table

立即更新。

Summary Cards 亦重新計算。

---

## 4.3 血糖標準值

Sidebar 顯示血糖參考值。

內容：

### 空腹血糖

正常人空腹 8 小時後：

70～99 mg/dL

---

### 餐前血糖

距離上一餐至少 4 小時：

70～100 mg/dL

---

### 餐後血糖

餐後 2 小時：

正常值 < 140 mg/dL

若 ≥140 mg/dL

視為偏高。

此區僅供參考。

不參與任何計算。

---

## 4.4 更新資料

提供：

【立即更新】

按鈕。

按下後：

重新讀取 Google Sheet。

重新計算所有分析。

重新整理 Dashboard。

另外：

當 Sidebar 條件改變時，

系統亦應自動重新整理。

不再提供固定更新頻率。

---

## 4.5 Google Sheet 狀態

顯示：

- Google Sheet 已連線
- 最後同步時間

若同步失敗：

顯示警告訊息。

---

# 5. Summary Cards

Summary Cards 永遠位於第一列。

依據目前 Sidebar 條件重新計算。

共三張：

---

## Card 1

平均血糖

顯示：

Average Blood Glucose

單位：

mg/dL

並顯示：

目前統計區間。

---

## Card 2

糖化血色素（估算）

依據平均血糖自動換算。

顯示：

Estimated HbA1c

單位：

%

同時顯示：

Estimated Average Glucose（eAG）。

---

## Card 3

TIR（Time In Range）

顯示：

目前區間內：

70~180 mg/dL

所佔百分比。

並顯示：

High %

Low %

In Range %

---

# 6. Blood Glucose Trend Chart

此圖為 Dashboard 主要視覺區域。

依據 Sidebar 條件更新。

支援：

- 日
- 週
- 月
- 季

---

## X Axis

依時間區間自動調整。

例如：

日：

24 小時

週：

7 天

月：

30 天

季：

90 天

---

## Y Axis

Blood Glucose（mg/dL）

---

## 血糖區間

背景分三層：

低血糖

<70 mg/dL

黃色

---

正常

70~180 mg/dL

綠色

---

偏高

>180 mg/dL

紅色

---

## 超標顯示

若血糖超過正常範圍：

折線節點：

紅色。

Tooltip：

顯示：

- 日期
- 時間
- 血糖值
- 分類
- 備註

---

# 7. Blood Glucose Record Table

Table 顯示 Google Sheet 原始資料。

欄位順序：

| 欄位 |
|------|
| 血糖量測日期時間 |
| 事件 |
| 量測血糖值 (mg/dL) |
| 備註1 |
| 備註2 |

支援：

- 排序
- 搜尋
- CSV 匯出

若血糖異常：

數值使用紅色。

---

# 8. Dashboard 更新規則

以下操作皆須重新整理 Dashboard：

- 切換日／週／月／季
- 修改日期區間
- 修改篩選條件
- 按下立即更新

更新內容：

- Summary Cards
- Trend Chart
- Detail Table

全部同步更新。

---

# 9. 色彩規範

Primary

Blue

Normal

Green

Warning

Orange

High

Red

Background

White

Card

Light Gray

---

# 10. 響應式設計

第一階段：

Desktop

1920

1600

1366

第二階段：

Tablet

第三階段：

Mobile

---

# 11. UI 設計原則

整體 Dashboard 必須符合：

- Information First
- One Screen Dashboard
- Zero Learning Cost
- Professional Medical Style
- Fast Loading
- Minimal Click
- High Readability
- Consistent Component Design

所有重要資訊應在第一個畫面即可看到，不需要捲動頁面。

# 16. 非功能需求 (Non-Functional Requirements)

## 啟動速度

目標：

3 秒內完成 Dashboard 啟動。

---

## 記憶體

Rust Backend：

< 100 MB

---

## 安裝

整個產品：

盡可能維持輕量。

---

## 相容性

支援：

* Windows 11
* Windows 10

Linux 與 macOS 為後續規劃。

---

# 17. 第一階段功能 (MVP)

* Google Sheets 讀取
* Dashboard 首頁
* 平均血糖
* 最高/最低
* 24 小時血糖折線圖
* 最近紀錄
* CLI 啟動
* CLI 更新
* CLI 系統檢查

---

# 18. 未來擴充方向

第二階段：

* AI 血糖分析
* 趨勢預測
* PDF 報表
* 匯出 CSV
* 多個 Google Sheet
* 多位使用者

第三階段：

* CGM 即時資料
* Apple Health / Google Fit 整合
* 醫師分享模式
* AI 健康建議
* Plugin 擴充機制

---

# 19. AI 開發原則

所有 AI Agent 在開發時應遵循：

1. 採用 Rust + React + Browser 架構。
2. Backend 與 Frontend 保持低耦合，以 REST API 溝通。
3. Google Sheet 為唯一資料來源，不建立資料庫。
4. 優先保證安裝、更新與啟動流程穩定，再逐步增加分析功能。
5. CLI 體驗應接近成熟開發工具（如 Docker、Git、Claude CLI），保持指令簡潔、一致且易於診斷。
6. 每個功能應具備可測試性，方便 AI Agent 逐步實作、驗證與迭代。

#20. Data Source Specification

## Google Sheet

目前第一階段資料來源為 Google Sheet。

Google Sheet 必須包含固定 Header。

系統依據 Header 名稱解析資料。

Header 名稱必須完全一致。

---

## Sheet Format

| 欄位順序 | Header 名稱 | 型別 | 必填 | 說明 |
|----------|------------|------|------|------|
| 1 | 血糖量測日期時間 | DateTime | ✓ | 血糖量測日期與時間 |
| 2 | 事件 | String | ✓ | 血糖量測事件 |
| 3 | 量測血糖值(mg/dl) | Number | ✓ | 血糖值 |
| 4 | 備註1 | String | | 備註資訊 |
| 5 | 備註2 | String | | 備註資訊 |

---

## Header Definition

Header 必須完全一致：

```
血糖量測日期時間
事件
量測血糖值(mg/dl)
備註1
備註2
```

Header 不可修改。

不得使用英文 Header。

不得增加空白。

Header 大小寫（中文）需完全一致。

---

## Data Example

| 血糖量測日期時間 | 事件 | 量測血糖值(mg/dl) | 備註1 | 備註2 |
|-----------------|------|------------------|--------|--------|
| 2026/07/07 07:30 | 空腹血糖 | 98 | 睡眠良好 | |
| 2026/07/07 12:30 | 午餐前 | 115 | | |
| 2026/07/07 14:20 | 午餐後 | 152 | 飯後2小時 | |

---

## Event Definition

目前支援：

- 空腹血糖
- 午餐前
- 午餐後
- 晚餐前
- 晚餐後
- 睡前

事件名稱需完全一致。

未定義事件：

系統應標示 Unknown Event。

不得參與統計分析。

---

## Date Format

建議格式：

```
yyyy/MM/dd HH:mm
```

例如：

```
2026/07/07 07:30
```

系統應同時支援：

```
yyyy-MM-dd HH:mm
```

解析失敗：

略過該筆資料。

並記錄 Error Log。

---

## Blood Glucose

單位：

mg/dL

資料型別：

Integer

合法範圍：

20 ~ 600

若超出範圍：

視為 Invalid Data。

不得參與分析。

---

## Missing Value

若：

日期

事件

血糖值

任一缺少：

此筆資料略過。

並記錄 Warning。

---

## Data Mapping

Google Sheet

↓

BloodGlucoseRecord

```
BloodGlucoseRecord

DateTime

Event

Glucose

Remark1

Remark2
```

所有後端分析皆使用 BloodGlucoseRecord。

不得直接操作 Google Sheet Row。