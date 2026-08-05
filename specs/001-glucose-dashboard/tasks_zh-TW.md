# Tasks：本機血糖儀表板（Local Glucose Dashboard）

**輸入（Input）**

來自 `specs/001-glucose-dashboard/` 的設計文件。

**前置需求（Prerequisites）**

- `plan.md`
- `spec.md`
- `research.md`
- `data-model.md`
- `contracts/`

**測試（Tests）**

由於《Constitution》要求每項功能皆可獨立驗證，因此本專案包含：

- Contract Tests（契約測試）
- Integration Tests（整合測試）
- Component Tests（元件測試）
- Browser Tests（瀏覽器測試）

**任務組織方式（Organization）**

所有 Task 依照 User Story 分組，使每個 User Story 都可以：

- 獨立開發
- 獨立測試
- 獨立驗收

---

# Phase 1：專案初始化（Setup，共用基礎建設）

**目的（Purpose）**

初始化整個專案所需的共用基礎架構，包括：

- Backend
- Frontend
- Installer
- Test Harness

---

- [ ] **T001** 初始化 Rust Backend 專案，建立套件設定與執行入口。

  **修改檔案：**

  - `backend/Cargo.toml`
  - `backend/src/main.rs`

---

- [ ] **T002** 初始化 React + Vite Frontend 專案，建立瀏覽器入口。

  **修改檔案：**

  - `frontend/package.json`
  - `frontend/index.html`
  - `frontend/src/main.tsx`

---

- [ ] **T003 [P]** 配置整個 Repository 共用的：

  - 程式碼格式化（Formatting）
  - Lint 規則
  - Type Checking
  - Build Output Ignore

  **修改檔案：**

  - `.editorconfig`
  - `rustfmt.toml`
  - `frontend/eslint.config.js`
  - `frontend/tsconfig.json`
  - `.gitignore`

---

- [ ] **T004 [P]** 建立 Backend、Frontend、測試資料（Fixture）以及 Browser Test 的測試架構。

  **建立目錄：**

  - `backend/tests/`
  - `frontend/tests/`
  - `tests/fixtures/`
  - `tests/e2e/`

---

- [ ] **T005 [P]** 建立 Windows 安裝程式骨架（Installer Scaffold）與操作文件。

  **修改檔案：**

  - `installer/install.ps1`
  - `installer/README.md`

---

# Phase 2：基礎建設（Foundational）

**目的（Purpose）**

完成所有 User Story 共用的基礎能力，包括：

- Domain Model
- Security
- Service
- Error Handling
- Test Foundation

---

## ⚠️ 重要（CRITICAL）

**在完成本階段之前，不可開始任何 User Story 的開發。**

---

- [ ] **T006** 定義以下 Domain Model：

  - `GlucoseRecord`
  - Source Events
  - Classification Context
  - Shared Domain Types

  **修改檔案：**

  - `backend/src/domain/mod.rs`
  - `backend/src/domain/records.rs`

---

- [ ] **T007** 實作：

  - Header 完全一致比對
  - 日期解析
  - 必填欄位驗證
  - 血糖值範圍驗證
  - Data Quality Issue Code

  **修改檔案：**

  - `backend/src/ingestion/sheet_parser.rs`
  - `backend/src/domain/data_quality.rs`

---

- [ ] **T008** 依照 `data-model.md` 實作情境式分類規則：

  - 空腹血糖
  - 餐前血糖
  - 餐後血糖
  - 睡前血糖

  **修改檔案：**

  - `backend/src/analysis/classification.rs`

---

- [ ] **T009** 實作分析統計：

  - 平均血糖
  - 最低血糖
  - 最高血糖
  - Estimated HbA1c
  - Estimated Average Glucose（eAG）
  - 各分類百分比統計

  **修改檔案：**

  - `backend/src/analysis/summary.rs`

---

- [ ] **T010** 實作版本化 Local Configuration，包括：

  - 載入
  - 儲存
  - Migration
  - Validation

  **修改檔案：**

  - `backend/src/config/model.rs`
  - `backend/src/config/store.rs`

---

- [ ] **T011** 實作作業系統安全憑證儲存（Credential Store）。

  功能包括：

  - Secure Credential Store Adapter
  - 隱藏敏感資訊的 Credential Status

  **修改檔案：**

  - `backend/src/auth/credential_store.rs`

---

- [ ] **T012** 建立共用錯誤處理機制，包括：

  - 穩定 Error Code
  - 繁體中文錯誤訊息
  - Structured Logging
  - Diagnostics Primitive

  **修改檔案：**

  - `backend/src/errors.rs`
  - `backend/src/diagnostics/mod.rs`
  - `backend/src/observability.rs`

---

- [ ] **T013** 建立 Local Service 基礎架構，包括：

  - Bootstrap
  - Middleware
  - Static Frontend
  - Route Registration

  **修改檔案：**

  - `backend/src/api/mod.rs`
  - `backend/src/api/router.rs`
  - `backend/src/main.rs`

---

- [ ] **T014** 建立 Frontend 共用型別，包括：

  - Service Client
  - Selection State
  - Synchronization State
  - Shared Response Models

  **修改檔案：**

  - `frontend/src/services/local_service.ts`
  - `frontend/src/state/dashboard_store.ts`
  - `frontend/src/types/`

---

- [ ] **T015** 建立完整測試資料（Fixtures），包括：

  - 正常資料
  - 格式錯誤資料
  - Threshold 邊界資料
  - Authentication 資料
  - Synchronization 資料

  **修改目錄：**

  - `backend/tests/fixtures/`
  - `frontend/tests/fixtures/`
  - `tests/fixtures/`

---

## Checkpoint（檢查點）

完成本階段後，應具備：

- 完整 Domain Rules
- 安全的設定管理
- Local Service 基礎架構
- 完整 Fixtures

完成後即可開始各 User Story 的獨立開發。
# Phase 3：User Story 1 - 安裝並連接 Dashboard（Priority：P1）🎯 MVP

**目標（Goal）**

讓第一次使用 Windows 的使用者能夠完成產品安裝、Google 驗證、驗證單一 Google Sheet、儲存安全設定，並成功開啟包含有效資料來源的 Dashboard。

**獨立驗證（Independent Test）**

於支援的 Windows 環境中，搭配預先準備好的 Google Sheet：

1. 執行安裝指令
2. 完成首次設定流程
3. 啟動產品
4. 驗證瀏覽器成功開啟 Dashboard
5. Dashboard 成功連線至 Google Sheet
6. 系統未保存任何明文 OAuth Secret

---

## User Story 1 測試（Tests）

- [ ] **T016 [P] [US1]** 新增 CLI Contract Tests，驗證：

  - `config`
  - 預設啟動流程
  - Exit Code
  - 敏感資訊遮罩（Redacted Output）
  - Retry 行為

  **修改檔案：**

  - `backend/tests/contract/cli_config.rs`

---

- [ ] **T017 [P] [US1]** 新增首次執行（First Run）整合測試，驗證：

  - Browser OAuth Authentication
  - Secure Credential Storage
  - Google Sheet 存取
  - 不完整設定應拒絕啟動

  **修改檔案：**

  - `backend/tests/integration/first_run_config.rs`

---

## User Story 1 實作（Implementation）

- [ ] **T018 [US1]** 實作 Desktop OAuth 驗證流程，包括：

  - Browser OAuth Flow
  - Callback Handling
  - Token Refresh
  - Secure Store Failure Handling

  **修改檔案：**

  - `backend/src/auth/oauth.rs`

---

- [ ] **T019 [US1]** 實作首次執行設定流程，包括：

  - Google Sheet Metadata
  - Credential Reference
  - Schema Validation

  **修改檔案：**

  - `backend/src/config/service.rs`

---

- [ ] **T020 [US1]** 依照 `contracts/local-service.md` 實作：

  - Configuration Status
  - Configure Operations

  **修改檔案：**

  - `backend/src/api/config_routes.rs`

---

- [ ] **T021 [US1]** 實作 CLI 指令：

  - `glucose-dashboard config`
  - 預設啟動流程
  - `glucose-dashboard start`

  **修改檔案：**

  - `backend/src/cli/config.rs`
  - `backend/src/cli/start.rs`

---

- [ ] **T022 [US1]** 實作 Windows 一鍵安裝，包括：

  - One-command Installation
  - 安裝至目前使用者目錄
  - PATH 設定
  - 安裝失敗回復（Rollback）

  **修改檔案：**

  - `installer/install.ps1`

---

- [ ] **T023 [US1]** 實作繁體中文首次使用介面，包括：

  - 初始設定畫面
  - Authentication Error 畫面
  - Loading 狀態
  - 第一次成功連線畫面

  **修改檔案：**

  - `frontend/src/components/setup/`
  - `frontend/src/components/layout/`
  - `frontend/src/app/App.tsx`

---

- [ ] **T024 [US1]** 新增 Browser End-to-End 測試，驗證：

  - 安裝完成後交接流程
  - 首次設定流程
  - Retry
  - Dashboard 開啟

  **修改檔案：**

  - `tests/e2e/first-run.spec.ts`

---

## Checkpoint（檢查點）

完成本階段後，第一次使用者應能夠：

- 完成安裝
- 完成設定
- 啟動產品
- 成功進入 Dashboard

且以上流程皆可獨立於後續分析功能完成。

---

# Phase 4：User Story 2 - 一眼掌握血糖摘要（Priority：P1）

**目標（Goal）**

依照設計圖（Reference Image）呈現 Dashboard，並同步顯示：

- Summary Cards
- Blood Glucose Trend Chart
- 初始 Blood Glucose Record Table

所有資料皆依目前選取資料同步更新。

---

**獨立驗證（Independent Test）**

載入測試資料（Fixture）後：

驗證：

- 三張 Summary Cards
- 情境式分類
- Trend Chart
- 異常血糖點
- 指定期間資料

皆能正確顯示且資料一致。

---

## User Story 2 測試（Tests）

- [ ] **T025 [P] [US2]** 新增 Analysis Unit Tests，驗證：

  - Summary Values
  - Empty Selection
  - Mixed Event Context
  - Threshold Classification

  **修改檔案：**

  - `backend/tests/unit/analysis_summary.rs`

---

- [ ] **T026 [P] [US2]** 新增 Frontend Component Tests，驗證：

  - Summary Cards
  - Chart State
  - Tooltip
  - Record Table 初始畫面

  **修改檔案：**

  - `frontend/tests/components/dashboard_summary.test.tsx`

---

## User Story 2 實作（Implementation）

- [ ] **T027 [US2]** 實作 Dashboard API 回傳內容，包括：

  - Selection
  - Summary
  - Trend Points
  - Table Rows
  - Synchronization Metadata
  - Data Quality Issues

  所有資料須以一次回應（Atomic Response）完成。

  **修改檔案：**

  - `backend/src/api/dashboard_routes.rs`

---

- [ ] **T028 [US2]** 實作三張 Summary Cards：

  - 平均血糖
  - Estimated HbA1c / eAG
  - Contextual TIR

  **修改檔案：**

  - `frontend/src/components/summary/SummaryCards.tsx`
  - `frontend/src/components/summary/MetricCard.tsx`

---

- [ ] **T029 [US2]** 實作 Blood Glucose Trend Chart，包括：

  - 情境式血糖曲線
  - Threshold 區域
  - Legend
  - 異常點樣式
  - Point Inspection

  **修改檔案：**

  - `frontend/src/components/trend/GlucoseTrendChart.tsx`
  - `frontend/src/components/trend/TrendTooltip.tsx`

---

- [ ] **T030 [US2]** 實作 Blood Glucose Record Table，包括：

  - Google Sheet 欄位順序
  - 異常血糖值顯示

  **修改檔案：**

  - `frontend/src/components/records/GlucoseRecordTable.tsx`

---

- [ ] **T031 [US2]** 依照 Reference Image 實作 Desktop Dashboard Layout，包括：

  - Header
  - Sidebar
  - Summary Row
  - Trend Panel
  - Table Panel

  **修改檔案：**

  - `frontend/src/components/layout/DashboardLayout.tsx`
  - `frontend/src/styles/dashboard.css`

---

- [ ] **T032 [US2]** 新增 Browser Smoke Tests，驗證：

  - Dashboard Layout
  - Summary Calculations
  - Chart Point Inspection
  - Empty State
  - 繁體中文介面文字

  **修改檔案：**

  - `tests/e2e/dashboard-summary.spec.ts`

---

## Checkpoint（檢查點）

完成本階段後，Dashboard 應提供完整的唯讀分析介面，包括：

- Summary Cards
- Trend Chart
- Record Table
- 與 `Dashboard Image.png` 相同的視覺配置

# Phase 5：User Story 3 - 篩選並重新整理分析結果（Priority：P1）

**目標（Goal）**

讓使用者能夠：

- 切換分析期間
- 切換事件分類
- 手動同步 Google Sheet

並且所有 Dashboard 視圖皆應同步更新。

若同步失敗，系統應清除目前顯示資料，以避免顯示過期（Stale）的分析結果。

---

## 獨立驗證（Independent Test）

使用跨越不同日期與不同事件的測試資料：

驗證：

1. 切換所有期間（Period）
2. 切換所有事件（Event Filter）
3. 手動重新同步 Google Sheet
4. 模擬同步失敗

確認：

- 所有 Dashboard 視圖同步更新
- 同步失敗時，畫面應清除既有分析資料並顯示錯誤狀態

---

## User Story 3 測試（Tests）

- [ ] **T033 [P] [US3]** 新增 Local Service Contract Tests，驗證：

  - Period Filters
  - Event Filters
  - Atomic Dashboard Response
  - Synchronization Failure Payload

  **修改檔案：**

  - `backend/tests/contract/dashboard_selection.rs`

---

- [ ] **T034 [P] [US3]** 新增 Integration Tests，驗證：

  - 成功同步
  - Invalid Row Diagnostics
  - 同步失敗清除資料
  - Last Successful Sync Metadata

  **修改檔案：**

  - `backend/tests/integration/synchronization.rs`

---

## User Story 3 實作（Implementation）

- [ ] **T035 [US3]** 實作分析條件選擇功能，包括：

  - 預設期間（Period Presets）
  - 自訂日期區間
  - Event Filters
  - 與搜尋無關的 Selection
  - 每筆資料分類統計

  **修改檔案：**

  - `backend/src/analysis/selection.rs`

---

- [ ] **T036 [US3]** 實作 Google Sheet 同步流程，包括：

  - Google Sheet Fetch
  - Row Validation
  - Issue Collection
  - Synchronization State Transition
  - 同步失敗時清除資料

  **修改檔案：**

  - `backend/src/ingestion/sync_service.rs`
  - `backend/src/api/sync_routes.rs`

---

- [ ] **T037 [US3]** 實作 Sidebar 控制元件，包括：

  - Period Controls
  - Custom Date Picker
  - Event Radio Filters
  - Refresh Button
  - Connection Status

  **修改檔案：**

  - `frontend/src/components/sidebar/TimePeriodControls.tsx`
  - `frontend/src/components/sidebar/EventFilters.tsx`
  - `frontend/src/components/sidebar/SyncStatus.tsx`

---

- [ ] **T038 [US3]** 實作 Dashboard State 管理，包括：

  - Filter Change
  - Loading State
  - Empty State
  - Synchronization Failure

  所有 Dashboard 資料需以 Atomic Replace 方式更新。

  **修改檔案：**

  - `frontend/src/state/dashboard_store.ts`
  - `frontend/src/services/dashboard_queries.ts`

---

- [ ] **T039 [US3]** 新增 Browser 測試，驗證：

  - 所有 Period Presets
  - Custom Date
  - Event Filters
  - Manual Refresh
  - Dashboard 同步更新
  - Synchronization Failure 後資料清除

  **修改檔案：**

  - `tests/e2e/dashboard-filters.spec.ts`

---

- [ ] **T040 [US3]** 新增效能測試，驗證：

  - Filter Response Time
  - Refresh Performance

  使用 Fixture Corpus 進行測試。

  **修改檔案：**

  - `backend/tests/integration/dashboard_performance.rs`
  - `frontend/tests/integration/dashboard_updates.test.tsx`

---

## Checkpoint（檢查點）

完成本階段後：

- Dashboard 篩選功能完整
- Google Sheet 同步機制完整
- 不會顯示過期（Stale）的分析資料
- 所有 Dashboard 視圖保持一致

---

# Phase 6：User Story 4 - 檢視並匯出原始資料（Priority：P2）

**目標（Goal）**

讓使用者可以：

- 搜尋資料
- 排序資料
- 檢視 Google Sheet 原始資料
- 匯出 CSV

以上操作皆不得影響 Summary Cards 與分析結果。

---

## 獨立驗證（Independent Test）

操作流程：

1. 搜尋資料
2. 排序資料
3. 驗證 Summary Cards 與 Trend Chart 不受影響
4. 匯出 CSV
5. 比對 CSV 與目前 Table 內容

確認：

- 欄位一致
- 資料一致
- 排序一致

---

## User Story 4 測試（Tests）

- [ ] **T041 [P] [US4]** 新增 Contract Tests，驗證：

  - Table Search
  - Sorting
  - Visible Row Selection
  - CSV Columns
  - Traditional Chinese Encoding

  **修改檔案：**

  - `backend/tests/contract/records_export.rs`

---

- [ ] **T042 [P] [US4]** 新增 Frontend Tests，驗證：

  - Search
  - Sorting

  僅影響 Table，不影響分析結果。

  **修改檔案：**

  - `frontend/tests/components/record_table_controls.test.tsx`

---

## User Story 4 實作（Implementation）

- [ ] **T043 [US4]** 實作 Backend Table Query，包括：

  - Projection
  - Search
  - Stable Sorting
  - CSV Export

  **修改檔案：**

  - `backend/src/api/records_routes.rs`
  - `backend/src/export/csv.rs`

---

- [ ] **T044 [US4]** 實作 Frontend Table，包括：

  - Search
  - Sort Controls
  - No Results
  - 異常值 Badge
  - CSV Export

  **修改檔案：**

  - `frontend/src/components/records/RecordTableToolbar.tsx`
  - `frontend/src/components/records/GlucoseRecordTable.tsx`

---

- [ ] **T045 [US4]** 完成 CSV 匯出功能，包括：

  - 繁體中文 Header
  - UTF-8 Encoding
  - Export Failure Diagnostics

  **修改檔案：**

  - `backend/src/export/csv.rs`
  - `backend/src/errors.rs`

---

- [ ] **T046 [US4]** 新增 Browser 測試，驗證：

  - Search
  - Sorting
  - No Results
  - Export Success
  - Export Failure

  **修改檔案：**

  - `tests/e2e/record-table.spec.ts`

---

## Checkpoint（檢查點）

完成本階段後：

- 使用者可搜尋資料
- 可排序資料
- 可匯出 CSV
- 所有分析結果保持不變
- Record Table 可獨立使用，不影響 Dashboard 分析功能
# Phase 7：User Story 5 - 每日重複使用 Dashboard（Priority：P2）

**目標（Goal）**

讓使用者在完成首次設定後，日常只需執行一次啟動指令即可使用 Dashboard。

系統應：

- 重複使用已儲存的設定
- 避免啟動多個本機服務實例
- 自動開啟或聚焦（Focus）已存在的 Dashboard 視窗

---

## 獨立驗證（Independent Test）

完成首次設定後：

1. 多次執行啟動指令
2. 驗證系統會重複使用既有設定
3. 驗證 Dashboard 能正常開啟或切換至既有視窗
4. 模擬 Google Sheet 無法存取

確認：

- 不需重新設定
- 不會產生重複的本機服務
- 系統能顯示清楚的錯誤訊息

---

## User Story 5 測試（Tests）

- [ ] **T047 [P] [US5]** 新增 Startup Integration Tests，驗證：

  - Reload Saved Configuration
  - Credential Refresh
  - Local Service Readiness
  - Google Sheet 無法存取時的啟動失敗

  **修改檔案：**

  - `backend/tests/integration/daily_start.rs`

---

- [ ] **T048 [P] [US5]** 新增 Browser Tests，驗證：

  - 重複啟動
  - Dashboard Focus / Navigation
  - Startup Error State

  **修改檔案：**

  - `tests/e2e/daily-start.spec.ts`

---

## User Story 5 實作（Implementation）

- [ ] **T049 [US5]** 實作啟動流程，包括：

  - Reload Configuration
  - Credential Retrieval
  - Readiness Check
  - Browser Launch

  **修改檔案：**

  - `backend/src/cli/start.rs`
  - `backend/src/runtime/startup.rs`

---

- [ ] **T050 [US5]** 實作 Single Instance 管理，包括：

  - Single Instance Detection
  - Reuse Local Service
  - Existing Dashboard Navigation

  **修改檔案：**

  - `backend/src/runtime/single_instance.rs`
  - `backend/src/runtime/browser.rs`

---

- [ ] **T051 [US5]** 實作 Frontend 啟動狀態，包括：

  - Startup Loading
  - Connection Failure
  - No Stale Data State

  **修改檔案：**

  - `frontend/src/components/layout/StartupState.tsx`
  - `frontend/src/state/dashboard_store.ts`

---

- [ ] **T052 [US5]** 撰寫日常使用文件，包括：

  - CLI 使用方式
  - 指令範例
  - 日常操作流程

  **修改檔案：**

  - `README.md`

---

## Checkpoint（檢查點）

完成本階段後：

- 使用者每天只需一個指令即可啟動 Dashboard
- 系統自動重用設定
- 不會啟動多個本機服務
- 不會顯示過期（Stale）資料

---

# Phase 8：User Story 6 - 系統診斷與安全更新（Priority：P2）

**目標（Goal）**

提供：

- 系統診斷（Doctor）
- Version 查詢
- 安全更新
- 更新失敗復原（Recovery）

且整個更新流程不得遺失：

- 使用者設定
- 已安裝版本

---

## 獨立驗證（Independent Test）

執行：

1. Doctor
2. Version
3. 模擬更新成功
4. 模擬更新失敗

確認：

- 設定仍保留
- 可回復上一個可用版本
- 不需重新設定 Dashboard

---

## User Story 6 測試（Tests）

- [ ] **T053 [P] [US6]** 新增 CLI Contract Tests，驗證：

  - Doctor Check Name
  - Exit Code
  - Version Output
  - 繁體中文診斷訊息

  **修改檔案：**

  - `backend/tests/contract/doctor_version.rs`

---

- [ ] **T054 [P] [US6]** 新增 Update Integration Tests，驗證：

  - Compatible Release
  - Configuration Preservation
  - Interrupted Download
  - Rollback

  **修改檔案：**

  - `backend/tests/integration/update_recovery.rs`

---

## User Story 6 實作（Implementation）

- [ ] **T055 [US6]** 實作 Doctor Health Checks，包括：

  - Login
  - Google Sheet
  - Network
  - Configuration
  - Cache
  - Dashboard

  **修改檔案：**

  - `backend/src/diagnostics/checks.rs`
  - `backend/src/cli/doctor.rs`

---

- [ ] **T056 [US6]** 實作 Update Service，包括：

  - Compatible Release Discovery
  - Download Verification
  - Staged Replacement
  - Configuration Preservation
  - Rollback

  **修改檔案：**

  - `backend/src/update/service.rs`
  - `backend/src/update/recovery.rs`

---

- [ ] **T057 [US6]** 實作 CLI 指令：

  - `glucose-dashboard update`
  - `glucose-dashboard version`

  包括：

  - Command Output
  - Exit Behavior

  **修改檔案：**

  - `backend/src/cli/update.rs`
  - `backend/src/cli/version.rs`

---

- [ ] **T058 [US6]** 新增繁體中文診斷與更新資訊，包括：

  - Update Status
  - Diagnostic Detail
  - Recovery Guidance
  - Local Service Error Response

  **修改檔案：**

  - `backend/src/diagnostics/report.rs`
  - `backend/src/errors.rs`

---

- [ ] **T059 [US6]** 新增 Browser 與 CLI Smoke Tests，驗證：

  - Doctor
  - Version
  - Update Success
  - Update Failure Recovery

  **修改檔案：**

  - `tests/e2e/maintenance.spec.ts`
  - `backend/tests/integration/maintenance.rs`

---

## Checkpoint（檢查點）

完成本階段後：

- 使用者可自行診斷常見問題
- 可安全更新系統
- 更新失敗可回復上一版本
- 不會遺失設定或安裝環境

---

# Phase 9：完善與跨功能驗證（Polish & Cross-Cutting Concerns）

**目的（Purpose）**

在正式發佈前，依據 Constitution 的品質要求，驗證整個產品的品質與完整 Quickstart 流程。

---

- [ ] **T060 [P]** 稽核無障礙（Accessibility）功能，包括：

  - Keyboard Navigation
  - Semantic Labels
  - Focus States
  - Contrast
  - 不依賴顏色的狀態提示

  **修改檔案：**

  - `frontend/src/components/`
  - `frontend/src/styles/accessibility.css`
  - `frontend/tests/accessibility/dashboard_a11y.test.tsx`

---

- [ ] **T061 [P]** 量測效能，包括：

  - Startup
  - Filter
  - Refresh
  - Memory Usage

  使用固定 Fixture 重複測試。

  **修改檔案：**

  - `tests/e2e/performance.spec.ts`
  - `backend/tests/integration/performance.rs`

---

- [ ] **T062 [P]** 進行安全性審查，包括：

  - Credential Handling
  - Logs
  - Error Payload
  - Local Configuration Permissions
  - Dependency Security

  **修改檔案：**

  - `backend/src/auth/`
  - `backend/src/config/`
  - `backend/src/observability.rs`
  - `SECURITY.md`

---

- [ ] **T063** 執行 `specs/001-glucose-dashboard/quickstart.md` 的所有流程，並將結果記錄於：

  - `specs/001-glucose-dashboard/quickstart-results.md`

---

- [ ] **T064** 更新所有使用者文件，包括：

  - Installation
  - Setup
  - CLI Commands
  - Troubleshooting
  - Release Documentation

  **修改檔案：**

  - `README.md`
  - `docs/`

---

## Checkpoint（檢查點）

完成本階段後：

- 全部 User Story 已完成
- 通過 Constitution 品質要求
- 通過 Quickstart 驗證
- 可以正式發佈 Release
# 相依性與執行順序（Dependencies & Execution Order）

## 各階段相依性（Phase Dependencies）

- **Phase 1：Setup**
  - 無任何相依性。
  - T001～T005 用於初始化各個獨立的專案區域。

- **Phase 2：Foundational**
  - 相依於 Setup。
  - 在完成之前，所有 User Story 均不得開始開發。

- **Phase 3：User Story 1**
  - 相依於 Foundational。
  - 為 MVP（Minimum Viable Product，最小可行產品）的第一個增量。

- **Phase 4：User Story 2**
  - 相依於 Foundational。
  - End-to-End 流程需使用 US1 的設定流程。
  - Analysis 與 UI 可先利用 Fixture Data 開發。

- **Phase 5：User Story 3**
  - 相依於 Foundational。
  - 與 US2 Dashboard 整合，完成同步更新流程。

- **Phase 6：User Story 4**
  - 相依於：
    - US2 的 Record Table
    - US3 的 Selection State

- **Phase 7：User Story 5**
  - 相依於 US1 的 Configuration 與 Startup。
  - 在完成 Foundational 後，可與 US2、US3 平行開發。

- **Phase 8：User Story 6**
  - 相依於：
    - US1 Configuration
    - US5 Daily Startup

  以驗證更新後仍能保留使用者設定。

- **Phase 9：Polish**
  - 相依於所有需要交付的 User Story 均已完成。

---

## User Story 相依圖

```text
Setup
    │
    ▼
Foundational
    ├────────► US1（MVP） ─────► US5 ─────► US6
    │
    ├────────► US2 ─────────────┐
    │                           │
    └────────► US3 ◄────────────┘
                    │
                    ▼
                   US4

所有 User Story 完成
          │
          ▼
       Polish
```

---

### 補充說明

US2 與 US3 的核心功能可在完成 Foundational 後，利用 Fixture Data 平行開發。

US4 則依賴：

- US2 的 Record Table
- US3 的 Selection Contract

US5 與 US6 則依賴：

- Configuration
- Startup

以確保更新與重啟流程安全。

---

## 每個 User Story 的開發順序（Within Each User Story）

每個 User Story 均遵循以下流程：

1. 先撰寫 Contract Tests 或 Integration Tests。
2. 驗證測試會因功能尚未實作而失敗（Fail First）。
3. 建立 Domain Models 與 State Types。
4. 建立 Services。
5. 建立 API Routes。
6. 建立 UI。
7. 執行 Browser Tests。
8. 通過 Checkpoint。

---

# 平行開發範例（Parallel Execution Examples）

## Setup 與 Foundational

可同時進行：

```text
T001 Backend Package        || T002 Frontend Package       || T003 Repository Tooling
T004 Test Harness           || T005 Installer Scaffold
T006 Domain Types           || T010 Config Model           || T011 Secure Credential Adapter
T012 Errors / Logging       || T015 Fixture Corpus
```

當以下基礎完成後即可開始：

- T007～T009
- T013～T014

---

## User Story 1

可平行進行：

```text
T016 CLI Contract Tests     || T017 First-run Integration Tests
T018 OAuth Flow             || T023 Setup UI
```

之後：

T019～T022 完成 Configuration 與 Service Boundary。

最後執行：

T024 Browser End-to-End Tests。

---

## User Story 2

可平行進行：

```text
T025 Analysis Tests         || T026 Frontend Component Tests
T028 Summary Cards          || T029 Trend Chart
```

之後：

完成：

- T027 Dashboard Response
- T030 Record Table
- T031 Dashboard Layout

最後：

T032 驗證完整 User Story。

---

## User Story 3

可平行進行：

```text
T033 Contract Tests             || T034 Synchronization Integration Tests
T035 Selection Service          || T037 Sidebar Controls
```

之後：

完成：

- T036 Synchronization Service
- T038 Dashboard State

最後：

- T039 Browser Tests
- T040 Performance Tests

---

## User Story 4

可平行進行：

```text
T041 Export Contract Tests      || T042 Table Control Tests
T043 Backend Projection         || T044 Frontend Controls
```

之後：

完成：

T045 CSV Export Encoding。

最後：

T046 Browser Validation。

---

## User Story 5

可平行進行：

```text
T047 Startup Integration Tests      || T048 Repeated-start Browser Tests
T049 Startup Orchestration          || T050 Single-instance / Browser Reuse
```

最後完成：

- T051 Startup UI
- T052 Documentation

---

## User Story 6

可平行進行：

```text
T053 Doctor / Version Tests     || T054 Update Recovery Tests
T055 Diagnostics                || T056 Update Service
```

最後完成：

- T057 CLI Commands
- T058 Traditional Chinese Messages
- T059 End-to-End Maintenance Validation

---

# 實作策略（Implementation Strategy）

## 第一階段：先完成 MVP（User Story 1）

建議開發流程：

1. 完成 Phase 1：Setup。
2. 完成 Phase 2：Foundational。
3. 完成 Phase 3：User Story 1。
4. 驗證：
   - 安裝
   - 第一次登入
   - Google Sheet 驗證
   - 啟動流程
   - Dashboard 開啟
5. 通過 Constitution 所要求的品質門檻後，再 Demo 或發布 MVP。

---

## 第二階段：逐步交付（Incremental Delivery）

依序完成：

1. US2：Dashboard Summary 與視覺介面。
2. US3：Filter、Refresh、同步更新與避免顯示過期資料。
3. US4：搜尋、排序與 CSV 匯出。
4. US5：每日啟動流程與 Single Instance。
5. US6：Doctor、Version、Update、Recovery。
6. 每次 Release 前皆執行 Phase 9。

每一個 User Story 都應保持：

- 可獨立測試
- 可獨立驗收
- 可獨立交付

---

# 備註（Notes）

- **[P]** 表示此 Task：
  - 修改不同的檔案集合（File Set）。
  - 不依賴本階段其他尚未完成的 Task。
  - 可與其他 `[P]` Task 平行開發。

- **[US1] ~ [US6]**
  - 對應 `spec.md` 中的六個 User Story。

- 每一個 Task 都至少指定一個具體的檔案路徑。
  - 並依照實作相依性排序。

- `tasks.md` **刻意不建立任何永久性的 Application Database**。
  - 本專案所有分析皆以 Google Sheet 為唯一資料來源。