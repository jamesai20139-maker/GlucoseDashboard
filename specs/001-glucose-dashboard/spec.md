# Feature Specification: Local Glucose Dashboard

**Feature Branch**: `001-glucose-dashboard`

**Created**: 2026-08-05

**Status**: Draft

**Input**: User description: "Please integrate Dashboard Image.png into the specification"

**Source**: [`doc/PRD.md`](../../doc/PRD.md)

**Visual Reference**: [`Dashboard Image.png`](../../doc/Dashboard%20Image.png)

## Clarifications

### Session 2026-08-05

- Q: 圖表應如何處理圖片中的 `140 mg/dL` 警示線與 PRD 定義的 `70–180 mg/dL` TIR 範圍？
  → A: 所有分析與圖表依量測情境使用標準：空腹 8 小時後為 70–99 mg/dL，餐前距離上一餐至少 4 小時為 70–100 mg/dL，餐後 2 小時達到或超過 140 mg/dL 視為偏高；不再以單一 70–180 mg/dL 區間作為全部判定。
- Q: 睡前血糖應套用哪一個既有標準？ → A: 睡前套用餐後 2 小時標準，達到或超過 140 mg/dL 視為偏高。
- Q: Google 登入憑證應如何保存，以便使用者重新啟動或更新後仍能連線？ → A: 使用作業系統安全憑證儲存區保存登入憑證。
- Q: 如果重新同步 Google Sheet 失敗，Dashboard 應如何處理目前已顯示的資料？ → A: 清除所有資料，只顯示同步失敗訊息與最後成功同步時間。
- Q: MVP 的 Dashboard 介面是否只需要支援繁體中文？ → A: MVP 僅支援繁體中文，必要技術名詞保留英文縮寫。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install and connect the dashboard (Priority: P1)

As a first-time user, I want to install the product with one command and connect it to
my Google Sheet so that I can begin reviewing glucose data without setting up a server,
database, or development environment.

**Why this priority**: Without a working first-run path, no other dashboard capability
can deliver value.

**Independent Test**: On a supported Windows computer with a prepared Google Sheet, a
tester can install the product, start it, complete the connection setup, and reach a
dashboard containing the Sheet data.

**Acceptance Scenarios**:

1. **Given** a supported Windows computer without the product installed, **When** the
   user runs the documented installation command, **Then** the product is installed in
   the user environment and the documented command becomes available.
2. **Given** the product is installed but has no saved connection, **When** the user
   starts it and completes the first-run setup, **Then** the product verifies access to
   the selected Google Sheet and opens the dashboard.
3. **Given** the user denies Google access or provides an unavailable Sheet, **When** the
   setup is submitted, **Then** the product explains the failure and provides a way to
   retry without silently creating an incomplete configuration.

### User Story 2 - See an at-a-glance glucose summary (Priority: P1)

As a person monitoring glucose, I want the most important information visible on the
first screen so that I can understand the selected period without reading every record.

**Why this priority**: Immediate comprehension is the primary value of the dashboard.

**Independent Test**: With valid records available, a tester opens the dashboard and
verifies that the summary, trend, and recent records are visible and internally
consistent for the default time period.

**Acceptance Scenarios**:

1. **Given** valid records are available, **When** the dashboard opens, **Then** it shows
   the selected period, average glucose, highest glucose, lowest glucose, estimated
   HbA1c with estimated average glucose, and the percentage of records within their
   applicable fasting, pre-meal, or post-meal reference range.
2. **Given** valid records are available, **When** the dashboard opens, **Then** it shows
   a time-based trend with glucose values, abnormal-value highlighting, and a detail
   table containing the records in the selected period. The first view uses the visual
   hierarchy shown in the reference image: a fixed header, left control sidebar, three
   summary cards, trend chart, and record table.
3. **Given** the selected period contains no valid records, **When** the dashboard is
   displayed, **Then** the summary and visual areas show a clear empty state rather than
   misleading zeros or stale results.

### User Story 3 - Filter and refresh the analysis (Priority: P1)

As a regular user, I want to change the time period and event category so that I can
inspect a specific part of my glucose history without manually recalculating anything.

**Why this priority**: Filtering is required for the dashboard to support daily, weekly,
monthly, quarterly, and event-specific review.

**Independent Test**: With records spanning multiple periods and events, a tester changes
the available filters and verifies that every affected view reflects the same filtered
set of records.

**Acceptance Scenarios**:

1. **Given** the dashboard is showing data, **When** the user selects day, week, month,
   quarter, or a custom date range, **Then** the summary cards, trend chart, and detail
   table update to that period.
2. **Given** the dashboard is showing data, **When** the user selects All, fasting,
   pre-meal, post-meal, or bedtime events, **Then** the summary cards, trend chart, and
   detail table update to the selected event category.
3. **Given** the user presses the immediate refresh control, **When** the Sheet has new
   records, **Then** the dashboard reloads the source data, recalculates the analysis,
   and displays the new last-updated time.
4. **Given** the user changes any filter, **When** the change is accepted, **Then** the
   dashboard refreshes without requiring a separate page reload.

### User Story 4 - Inspect and export source records (Priority: P2)

As a user reviewing individual measurements, I want to search, sort, and export the
records shown by the current selection so that I can inspect details or use them outside
the dashboard.

**Why this priority**: Summary and trend views explain the overall state, while the
record list provides traceability back to the user's source data.

**Independent Test**: With multiple records visible, a tester searches and sorts the
table, verifies that abnormal values remain identifiable, and exports the visible data.

**Acceptance Scenarios**:

1. **Given** records are visible, **When** the user enters a search term, **Then** the
   table shows only matching date/time, event, glucose, or remark content.
2. **Given** records are visible, **When** the user sorts the table, **Then** the displayed
   rows change order without changing the active analysis selection.
3. **Given** a filtered record set is visible, **When** the user selects CSV export,
   **Then** the product exports the currently visible records with the same columns and
   values shown in the table.

### User Story 5 - Reuse the dashboard every day (Priority: P2)

As a returning user, I want one familiar command to open the dashboard and preserve my
settings so that daily review takes minimal effort.

**Why this priority**: The product is intended for repeated personal use, not a one-time
data inspection.

**Independent Test**: After first-run setup, a tester starts the product again and
verifies that the saved connection is reused, the dashboard opens, and no setup is
requested again.

**Acceptance Scenarios**:

1. **Given** a valid saved configuration exists, **When** the user runs the normal start
   command, **Then** the product checks the configuration, starts the local dashboard,
   and opens it in the system browser.
2. **Given** the dashboard is already open, **When** the user starts the product again,
   **Then** the product focuses or navigates to the existing dashboard instead of
   creating a confusing second daily workflow.
3. **Given** the Sheet cannot be reached during startup, **When** the dashboard opens,
   **Then** it clears the summary, trend, and record views, and displays a connection
   warning with the last successful synchronization information, if available.

### User Story 6 - Diagnose and update safely (Priority: P2)

As a user who is not a developer, I want system checks and safe updates so that I can
resolve common problems and receive new versions without losing my Sheet configuration.

**Why this priority**: One-command operation is a core product promise and must remain
safe over the product lifecycle.

**Independent Test**: A tester runs each diagnostic check, simulates an available update,
and verifies that the update preserves configuration and reports its result clearly.

**Acceptance Scenarios**:

1. **Given** the product is installed, **When** the user runs the system-check command,
   **Then** the product reports the status of the Google login, Sheet access, network,
   configuration, local cache, and dashboard startup checks.
2. **Given** a newer compatible version is available, **When** the user runs the update
   command, **Then** the product installs the newer version, preserves the saved Sheet
   configuration, and reports completion.
3. **Given** an update download or installation fails, **When** the update ends, **Then**
   the existing usable installation and configuration remain available, and the user
   receives a recovery-oriented error message.
4. **Given** the user runs the version command, **When** the command completes, **Then**
   it reports the installed product version without starting the dashboard.

### Edge Cases

- If the Sheet header does not exactly match the required contract, the product MUST
  stop analysis, identify the mismatch, and explain that the source format must be
  corrected.
- If a row is missing its date/time, event, or glucose value, the product MUST exclude
  that row from analysis and record a warning visible through diagnostics or logs.
- If a date cannot be parsed using a supported format, the product MUST exclude that
  row and record an error without preventing valid rows from being analyzed.
- If a glucose value is outside 20–600 mg/dL, the product MUST classify the row as
  invalid and exclude it from analysis.
- If an event is not one of the supported event names, the product MUST label it as
  `Unknown Event` and exclude it from statistical calculations while preserving the row
  for data-quality visibility.
- If a refresh fails, the dashboard MUST show a warning and last synchronization time,
  clear the summary, trend, and record views, and MUST NOT present stale data as current.
- If a selected period or event filter has no valid data, all affected views MUST show
  the same empty state.
- If the user runs the product on an unsupported operating system, the product MUST
  identify the unsupported environment and provide the supported-platform guidance.
- If a search returns no matching records, the table MUST show a no-results state without
  changing the summary or trend calculations.
- If CSV export cannot be completed, the product MUST explain the failure and leave the
  current table and analysis unchanged.
- If the operating system's secure credential store is unavailable, the product MUST
  explain that secure sign-in storage is unavailable and MUST require re-authentication
  rather than saving credentials in plain text.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The product MUST run as a local dashboard without requiring the user to
  operate a separate server or install a database.
- **FR-002**: The product MUST provide a one-command installation flow for Windows 10
  and Windows 11 that installs the product for the current user and makes its normal
  command available.
- **FR-003**: The product MUST provide a normal start command that reads saved settings,
  validates access, starts the local dashboard, and opens it in the system browser.
- **FR-004**: The product MUST provide commands for start, update, configuration,
  system checks, and version reporting with consistent names and user-readable output.
- **FR-005**: The product MUST support first-run configuration of the Google account and
  the Google Sheet needed for analysis.
- **FR-006**: The product MUST save the minimum non-sensitive configuration needed for
  subsequent use and MUST preserve it across compatible updates. Google login
  credentials or tokens MUST be stored only in the supported operating system's secure
  credential store and MUST NOT be written to a plain-text configuration file.
- **FR-007**: The product MUST read glucose records from the configured Google Sheet and
  MUST treat that Sheet as the authoritative source of records.
- **FR-008**: The product MUST require the exact following Sheet headers, in this order:
  `血糖量測日期時間`, `事件`, `量測血糖值(mg/dl)`, `備註1`, `備註2`.
- **FR-009**: The product MUST support date/time values in `yyyy/MM/dd HH:mm` and
  `yyyy-MM-dd HH:mm` formats.
- **FR-010**: The product MUST support glucose values as integers in the inclusive range
  20–600 mg/dL.
- **FR-011**: The product MUST recognize these event names exactly: `空腹血糖`, `午餐前`,
  `午餐後`, `晚餐前`, `晚餐後`, and `睡前`.
- **FR-012**: The product MUST convert each valid source row into a consistent glucose
  record containing date/time, event, glucose value, and the two optional remarks before
  calculating any analysis.
- **FR-013**: The product MUST exclude incomplete, unparsable, out-of-range, and unknown
  event records from statistical calculations and MUST record the applicable warning or
  error.
- **FR-014**: The product MUST display the source connection status and last successful
  synchronization time.
- **FR-015**: The dashboard MUST display the system name, last update time, and a manual
  refresh control in the header.
- **FR-016**: The dashboard MUST provide day, week, month, quarter, and custom date-range
  filters.
- **FR-017**: The dashboard MUST provide event filters for All, fasting, pre-meal,
  post-meal, and bedtime records, including the corresponding supported meal events.
- **FR-018**: Every accepted filter change MUST recalculate and synchronously update the
  summary cards, trend chart, and detail table.
- **FR-019**: The dashboard MUST display average, highest, and lowest glucose for the
  active selection, with the unit mg/dL and the selected period.
- **FR-020**: The dashboard MUST display estimated HbA1c and estimated average glucose
  as clearly labeled estimates derived from the active selection.
- **FR-021**: The dashboard MUST classify readings using the applicable measurement
  context: fasting after 8 hours MUST use 70–99 mg/dL as the reference range, pre-meal
  readings after at least 4 hours MUST use 70–100 mg/dL, and post-meal or bedtime
  readings MUST use the 2-hour standard where values at or above 140 mg/dL are high.
  The dashboard MUST display the percentage within the applicable reference range and
  the low/high percentages for the active selection; it MUST NOT use one universal
  70–180 mg/dL rule.
- **FR-022**: The trend visualization MUST adjust its time scale to the selected period,
  show glucose values in mg/dL, and distinguish low, applicable-reference-range, and
  high readings according to the measurement context.
- **FR-023**: The trend visualization MUST highlight values outside the applicable
  measurement-context standard and MUST provide date, time, value, event, and remarks
  when a value is inspected.
- **FR-024**: The detail table MUST show source records in this order: date/time, event,
  glucose value, remark 1, and remark 2.
- **FR-025**: The detail table MUST support sorting and searching, and MUST visually
  distinguish abnormal glucose values.
- **FR-026**: The sidebar MUST show these reference standards: fasting after 8 hours,
  70–99 mg/dL; pre-meal after at least 4 hours, 70–100 mg/dL; and post-meal or bedtime
  at 2 hours, values at or above 140 mg/dL classified as high. These standards MUST be
  used for chart and summary classification, and the product MUST label them as
  informational health references rather than medical advice.
- **FR-027**: The dashboard MUST provide a clear empty state when the active selection
  contains no valid records.
- **FR-028**: The product MUST provide a system-check command that reports the status of
  login, Sheet access, network access, configuration, cache, and dashboard startup.
- **FR-029**: The update command MUST check for a newer compatible release, install it
  when available, and preserve the user's saved configuration.
- **FR-030**: The product MUST report update failures without removing the last usable
  installation.
- **FR-031**: The product MUST provide a version command that reports the installed
  version and does not open the dashboard.
- **FR-032**: The product MUST clearly state that the dashboard is for glucose analysis
  and visualization and is not a medical diagnostic or treatment system.
- **FR-043**: When a Google Sheet synchronization fails, the product MUST clear all
  analysis values, chart points, and record rows from the active dashboard view, MUST
  show the synchronization error, and MUST show the last successful synchronization
  time when one exists.
- **FR-044**: All MVP user-facing interface labels, validation messages, empty states,
  warnings, and error messages MUST be written in Traditional Chinese. Established
  technical terms and abbreviations such as TIR, HbA1c, mg/dL, and CSV MAY remain in
  their standard form; language switching is outside the MVP.
- **FR-033**: The dashboard MUST use the reference image's information hierarchy: a
  persistent left sidebar, a top header, a main content area, a summary-card row, a
  trend-chart panel, and a record-table panel.
- **FR-034**: The left sidebar MUST present, in order, the time-period controls, custom
  date selection, immediate refresh control, synchronization status, event filters, and
  informational glucose reference ranges.
- **FR-035**: The time-period controls MUST provide visually distinct selections for day,
  week, month, and quarter, and the custom date range MUST show both selected dates in a
  single control.
- **FR-036**: The header MUST show the product identity on the left and provide reserved
  theme and user/settings controls on the right; the header MUST NOT contain analysis
  values.
- **FR-037**: The summary area MUST show three visually distinct cards for average
  glucose, estimated HbA1c, and TIR (percentage within the applicable reference range),
  with each card showing its value, unit, and a clear label. A comparison with the
  previous comparable period MAY be shown only when that comparison is available.
- **FR-038**: The trend panel MUST show its title, a period selector, a time-based line
  chart, context-sensitive threshold regions, and a legend explaining low,
  reference-range, high, and abnormal readings.
- **FR-039**: The trend chart MUST use distinct visual treatment for abnormal points and
  MUST retain the detailed inspection content defined by FR-023.
- **FR-040**: The record-table panel MUST show its title, search control, and CSV export
  control above the table, and MUST preserve the source-column order defined by FR-024.
- **FR-041**: The dashboard MUST use the reference image's light, high-contrast visual
  presentation as the MVP default, with blue as the primary interaction color, green for
  normal values, orange/yellow for warnings or low values, red for high values, and
  white or light gray surfaces.
- **FR-042**: The dashboard MUST keep the summary cards, trend chart, and record table
  visible in the primary desktop view without requiring horizontal scrolling at the
  supported desktop widths.

### Key Entities

- **Glucose Record**: A valid measurement with date/time, one supported event, an integer
  glucose value in mg/dL, and zero or more remarks.
- **Source Sheet Contract**: The exact five-column structure and allowed values used to
  interpret the Google Sheet.
- **Analysis Selection**: The active date range and event filter that determine which
  valid records appear in all dashboard views.
- **Analysis Summary**: Average, minimum, maximum, estimated HbA1c, estimated average
  glucose, and low/in-range/high percentages calculated from the active selection.
- **Synchronization State**: The connection status, last successful synchronization
  time, and any warning or error from the most recent refresh.
- **Local Configuration**: The saved non-sensitive information required to reconnect to
  the user's Google Sheet and preserve normal operation after restart or update; the
  associated Google login credential is held separately by the operating system's secure
  credential store.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A first-time user on Windows 10 or Windows 11 can go from installation to
  an opened dashboard with connected data in 10 minutes or less, without installing a
  database or writing application code.
- **SC-002**: After configuration is complete, the dashboard opens and displays the
  initial valid data set within 3 seconds under the supported normal-use scenario.
- **SC-003**: For every supported filter change, 100% of the summary, trend, and detail
  views reflect the same active selection, with no stale view remaining visible.
- **SC-004**: In a usability test, at least 90% of first-time testers can identify the
  average glucose, time-in-range percentage, and last synchronization status on their
  first dashboard visit without assistance.
- **SC-005**: In a data-quality test, 100% of incomplete, unparsable, out-of-range, and
  unknown-event rows are excluded from statistics and reported with the appropriate
  data-quality status; valid readings are classified using the applicable fasting,
  pre-meal, or post-meal standard.
- **SC-006**: A compatible update preserves 100% of the user's saved Sheet connection
  settings and does not require first-run setup again.
- **SC-007**: In a normal supported backend run, memory usage remains below 100 MB and
  the installation remains lightweight enough for ordinary personal use.
- **SC-008**: At least 90% of tested users can complete the daily flow—start the product,
  inspect the current period, and refresh data—without consulting developer-oriented
  documentation.
- **SC-009**: In a usability test using the reference layout, at least 90% of users can
  locate the time filter, event filter, refresh control, summary cards, trend chart, and
  record table without assistance.
- **SC-010**: A CSV export contains exactly the records currently visible in the table,
  with 100% of displayed columns and values preserved.
- **SC-011**: In a content review of the MVP, 100% of user-facing interface text and
  messages are in Traditional Chinese, except for the explicitly allowed technical
  terms and abbreviations.

## Assumptions

- Version 1 targets Windows 10 and Windows 11; Linux, macOS, tablet, and mobile support
  are outside the MVP scope.
- The user has a Google account and permission to read the configured Sheet, and the
  Sheet contains the exact five-column contract defined in FR-008.
- The supported Windows environment provides an operating-system secure credential store;
  the product will not fall back to plain-text credential storage if it is unavailable.
- The later Data Source Specification in `doc/PRD.md` is authoritative where earlier PRD
  sections use inconsistent labels; therefore the source field is named `事件`, not
  `量測節點`.
- The MVP refresh model is event-driven: startup, manual refresh, and filter changes
  update the dashboard. Fixed-interval polling is outside the MVP unless separately
  specified.
- The supplied `Dashboard Image.png` is the authoritative visual reference for MVP
  composition, spacing priorities, control placement, color semantics, and component
  hierarchy; exact pixel dimensions and icon artwork may vary.
- Glucose classification is context-sensitive: fasting uses 70–99 mg/dL after 8 hours,
  pre-meal uses 70–100 mg/dL after at least 4 hours, and post-meal or bedtime uses the
  2-hour threshold where values at or above 140 mg/dL are high. The former universal
  70–180 mg/dL TIR assumption is superseded by this clarification.
- A single local user and one configured Google Sheet are supported in the MVP. Multiple
  users and multiple Sheets are future extensions.
- MVP interface content is Traditional Chinese only; multilingual support and language
  switching are future extensions.
- Reference glucose ranges are informational product guidance and are not medical advice;
  the product does not diagnose, prescribe, or replace professional care.
- When the PRD does not specify an exact HbA1c conversion equation, the product will use
  one documented and consistently applied conversion and label its output as an estimate.
- PDF reports, AI analysis, trend prediction, CGM integration, health-platform
  integration, physician sharing, and plugin support are future-phase features and are
  excluded from this MVP specification. CSV export is included because it is shown in
  the supplied dashboard reference and specified in the record-table requirements.
- The product will display friendly, actionable messages for authentication, connection,
  parsing, update, and unsupported-environment failures while retaining diagnostic detail
  for troubleshooting.
