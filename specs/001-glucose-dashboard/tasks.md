# Tasks: Local Glucose Dashboard

**Input**: Design documents from `specs/001-glucose-dashboard/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md),
[research.md](./research.md), [data-model.md](./data-model.md),
[contracts/](./contracts/)

**Tests**: Included because the Constitution requires independently verifiable behavior
and the design defines contract, integration, component, and browser validation.

**Organization**: Tasks are grouped by user story to enable independent implementation
and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the backend, frontend, installer, and test harness structure.

- [X] T001 Initialize the Rust backend package and executable entry point in `backend/Cargo.toml` and `backend/src/main.rs`
- [X] T002 Initialize the React/Vite frontend package and browser entry point in `frontend/package.json`, `frontend/index.html`, and `frontend/src/main.tsx`
- [X] T003 [P] Configure repository-wide formatting, linting, type checking, and ignored build output in `.editorconfig`, `rustfmt.toml`, `frontend/eslint.config.js`, `frontend/tsconfig.json`, and `.gitignore`
- [ ] T004 [P] Create backend, frontend, fixture, and browser-test harnesses in `backend/tests/`, `frontend/tests/`, `tests/fixtures/`, and `tests/e2e/`
- [X] T005 [P] Create the Windows installation packaging scaffold and operator notes in `installer/install.ps1` and `installer/README.md`

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the shared domain, security, service, error, and test foundations
that every user story depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T006 Define `GlucoseRecord`, source events, classification contexts, and shared domain types in `backend/src/domain/mod.rs` and `backend/src/domain/records.rs`
- [X] T007 Implement exact-header parsing, date parsing, required-field validation, glucose-range validation, and data-quality issue codes in `backend/src/ingestion/sheet_parser.rs` and `backend/src/domain/data_quality.rs`
- [X] T008 Implement context-sensitive fasting, pre-meal, post-meal, and bedtime classification rules in `backend/src/analysis/classification.rs` using the rules in `data-model.md`
- [X] T009 Implement average, minimum, maximum, estimated HbA1c/eAG, and context-specific percentage calculations in `backend/src/analysis/summary.rs`
- [ ] T010 Implement versioned non-sensitive local configuration loading, saving, migration, and validation in `backend/src/config/model.rs` and `backend/src/config/store.rs`
- [ ] T011 Implement the operating-system secure credential-store adapter and redacted credential status in `backend/src/auth/credential_store.rs`
- [X] T012 Implement stable error codes, Traditional Chinese user messages, structured logging, and diagnostics primitives in `backend/src/errors.rs`, `backend/src/diagnostics/mod.rs`, and `backend/src/observability.rs`
- [X] T013 Create the local service bootstrap, middleware, static frontend serving, and route registration in `backend/src/api/mod.rs`, `backend/src/api/router.rs`, and `backend/src/main.rs`
- [X] T014 Create typed frontend service-client, selection-state, synchronization-state, and shared response models in `frontend/src/services/local_service.ts`, `frontend/src/state/dashboard_store.ts`, and `frontend/src/types/`
- [ ] T015 Add canonical valid, malformed, threshold, authentication, and synchronization fixtures in `backend/tests/fixtures/`, `frontend/tests/fixtures/`, and `tests/fixtures/`

**Checkpoint**: Domain rules, secure configuration, local-service wiring, and test
fixtures are ready; user stories can now be implemented as independent increments.

## Phase 3: User Story 1 - Install and connect the dashboard (Priority: P1) 🎯 MVP

**Goal**: Let a first-time Windows user install the product, authenticate, validate one
Google Sheet, save safe configuration, and open a dashboard containing valid source data.

**Independent Test**: On a supported Windows machine with a prepared Sheet, run the
installation command, complete first-run setup, start the product, and verify that the
browser opens with connected source data and no plaintext OAuth secret.

### Tests for User Story 1

- [ ] T016 [P] [US1] Add CLI contract tests for `config`, default start, exit codes, redacted output, and retry behavior in `backend/tests/contract/cli_config.rs`
- [ ] T017 [P] [US1] Add first-run integration tests for browser authentication, secure credential storage, Sheet access, and incomplete-configuration rejection in `backend/tests/integration/first_run_config.rs`

### Implementation for User Story 1

- [ ] T018 [US1] Implement desktop OAuth browser flow, callback handling, token refresh, and secure-store failure behavior in `backend/src/auth/oauth.rs`
- [ ] T019 [US1] Implement first-run configuration orchestration for Sheet metadata, credential references, and schema validation in `backend/src/config/service.rs`
- [X] T020 [US1] Implement configuration status and configure operations from `contracts/local-service.md` in `backend/src/api/config_routes.rs`
- [ ] T021 [US1] Implement `glucose-dashboard config`, default start, and `start` command orchestration in `backend/src/cli/config.rs` and `backend/src/cli/start.rs`
- [ ] T022 [US1] Implement one-command Windows installation, current-user placement, PATH setup, and failure rollback in `installer/install.ps1`
- [ ] T023 [US1] Implement Traditional Chinese setup, authentication-error, loading, and first-connected states in `frontend/src/components/setup/`, `frontend/src/components/layout/`, and `frontend/src/app/App.tsx`
- [ ] T024 [US1] Add browser end-to-end coverage for install handoff, first-run configuration, retry, and dashboard opening in `tests/e2e/first-run.spec.ts`

**Checkpoint**: A first-time user can install, configure, start, and reach a connected
dashboard independently of the later analysis and reporting enhancements.

## Phase 4: User Story 2 - See an at-a-glance glucose summary (Priority: P1)

**Goal**: Render the reference-image dashboard with synchronized summary cards, trend
visualization, and initial record table for valid selected data.

**Independent Test**: With fixture records loaded, open the dashboard and verify the
three cards, context-sensitive classifications, trend chart, abnormal points, and
selected-period records are visible and internally consistent.

### Tests for User Story 2

- [ ] T025 [P] [US2] Add analysis unit tests for summary values, empty selections, mixed event contexts, and threshold classification in `backend/tests/unit/analysis_summary.rs`
- [ ] T026 [P] [US2] Add frontend component tests for summary cards, chart states, tooltips, and initial record table rendering in `frontend/tests/components/dashboard_summary.test.tsx`

### Implementation for User Story 2

- [X] T027 [US2] Implement atomic dashboard response assembly for selection, summary, trend points, table rows, sync metadata, and quality issues in `backend/src/api/dashboard_routes.rs`
- [X] T028 [US2] Implement the three summary cards for average glucose, estimated HbA1c/eAG, and contextual TIR in `frontend/src/components/summary/SummaryCards.tsx` and `frontend/src/components/summary/MetricCard.tsx`
- [X] T029 [US2] Implement the context-sensitive trend chart, threshold regions, legend, abnormal point styling, and point inspection in `frontend/src/components/trend/GlucoseTrendChart.tsx` and `frontend/src/components/trend/TrendTooltip.tsx`
- [X] T030 [US2] Implement the initial glucose record table with source-column order and abnormal-value presentation in `frontend/src/components/records/GlucoseRecordTable.tsx`
- [X] T031 [US2] Implement the reference-image desktop layout, header, sidebar shell, summary row, trend panel, and table panel in `frontend/src/components/layout/DashboardLayout.tsx` and `frontend/src/styles/dashboard.css`
- [ ] T032 [US2] Add browser smoke coverage for the initial dashboard hierarchy, summary calculations, chart point inspection, empty state, and Traditional Chinese labels in `tests/e2e/dashboard-summary.spec.ts`

**Checkpoint**: The dashboard provides a complete read-only analysis view for a valid
selection, including the visual hierarchy from `Dashboard Image.png`.

## Phase 5: User Story 3 - Filter and refresh the analysis (Priority: P1)

**Goal**: Let users change periods or event filters and manually synchronize the Sheet,
with all dashboard views updating atomically and failures clearing displayed data.

**Independent Test**: With records spanning periods and events, change every period and
event filter, refresh with new data, then simulate failure and verify synchronized
updates or the required cleared-data error state.

### Tests for User Story 3

- [ ] T033 [P] [US3] Add local-service contract tests for period filters, event filters, atomic dashboard responses, and synchronization failure payloads in `backend/tests/contract/dashboard_selection.rs`
- [ ] T034 [P] [US3] Add integration tests for successful refresh, invalid-row diagnostics, failure clearing, and last-successful-sync metadata in `backend/tests/integration/synchronization.rs`

### Implementation for User Story 3

- [X] T035 [US3] Implement period presets, custom date ranges, event filters, search-independent selections, and per-record classification aggregation in `backend/src/analysis/selection.rs`
- [ ] T036 [US3] Implement Google Sheet fetch, row validation, issue collection, synchronization transitions, and failure clearing in `backend/src/ingestion/sync_service.rs` and `backend/src/api/sync_routes.rs`
- [ ] T037 [US3] Implement sidebar period controls, custom date picker, event radio filters, refresh button, and connection status in `frontend/src/components/sidebar/TimePeriodControls.tsx`, `frontend/src/components/sidebar/EventFilters.tsx`, and `frontend/src/components/sidebar/SyncStatus.tsx`
- [ ] T038 [US3] Implement atomic dashboard state replacement for filter changes, loading states, empty states, and synchronization failures in `frontend/src/state/dashboard_store.ts` and `frontend/src/services/dashboard_queries.ts`
- [ ] T039 [US3] Add browser coverage for all period presets, custom dates, event filters, manual refresh, synchronized view updates, and cleared-data failure behavior in `tests/e2e/dashboard-filters.spec.ts`
- [ ] T040 [US3] Add performance tests for filter response and refresh handling against the fixture corpus in `backend/tests/integration/dashboard_performance.rs` and `frontend/tests/integration/dashboard_updates.test.tsx`

**Checkpoint**: Selection and synchronization behavior is complete, deterministic, and
safe against stale-data presentation.

## Phase 6: User Story 4 - Inspect and export source records (Priority: P2)

**Goal**: Let users search, sort, inspect, and export the currently visible source rows
without changing aggregate calculations.

**Independent Test**: Search and sort a filtered record set, verify cards and chart remain
unchanged, then export CSV and compare its rows and columns with the visible table.

### Tests for User Story 4

- [ ] T041 [P] [US4] Add contract tests for table search, sorting, visible-row selection, CSV columns, and Traditional Chinese encoding in `backend/tests/contract/records_export.rs`
- [ ] T042 [P] [US4] Add frontend tests proving search and sort affect only table presentation and no-results state in `frontend/tests/components/record_table_controls.test.tsx`

### Implementation for User Story 4

- [ ] T043 [US4] Implement table query projection, search matching across visible fields, stable sorting, and CSV export serialization in `backend/src/api/records_routes.rs` and `backend/src/export/csv.rs`
- [ ] T044 [US4] Implement record-table search, sort controls, no-results state, abnormal badges, and CSV export action in `frontend/src/components/records/RecordTableToolbar.tsx` and `frontend/src/components/records/GlucoseRecordTable.tsx`
- [ ] T045 [US4] Add Traditional Chinese CSV header, UTF-8 export handling, and export-failure diagnostics in `backend/src/export/csv.rs` and `backend/src/errors.rs`
- [ ] T046 [US4] Add browser coverage for searching, sorting, no-results behavior, export success, and export failure in `tests/e2e/record-table.spec.ts`

**Checkpoint**: Record inspection and export are independently usable without changing
the active analysis selection or summary calculations.

## Phase 7: User Story 5 - Reuse the dashboard every day (Priority: P2)

**Goal**: Make the normal start command reuse safe configuration, avoid confusing
duplicate local sessions, and open the existing dashboard reliably.

**Independent Test**: After first-run setup, start the product repeatedly and verify
that it reuses configuration, opens or focuses the dashboard, and shows a clear error
state when the Sheet is unavailable.

### Tests for User Story 5

- [ ] T047 [P] [US5] Add startup integration tests for saved configuration reload, credential refresh, local service readiness, and unavailable-Sheet startup failure in `backend/tests/integration/daily_start.rs`
- [ ] T048 [P] [US5] Add browser tests for repeated start behavior, dashboard focus/navigation, and startup error state in `tests/e2e/daily-start.spec.ts`

### Implementation for User Story 5

- [ ] T049 [US5] Implement startup configuration reload, credential retrieval, readiness checks, and browser launch orchestration in `backend/src/cli/start.rs` and `backend/src/runtime/startup.rs`
- [ ] T050 [US5] Implement single-instance detection, local service reuse, and existing-dashboard navigation in `backend/src/runtime/single_instance.rs` and `backend/src/runtime/browser.rs`
- [ ] T051 [US5] Implement startup loading, connection failure, and no-stale-data states in `frontend/src/components/layout/StartupState.tsx` and `frontend/src/state/dashboard_store.ts`
- [ ] T052 [US5] Add daily-use documentation and command examples matching the CLI contract in `README.md`

**Checkpoint**: A configured user can use one familiar command for daily review without
repeating setup or seeing stale data as current.

## Phase 8: User Story 6 - Diagnose and update safely (Priority: P2)

**Goal**: Provide system diagnostics, version reporting, compatible updates, and recovery
behavior without losing the last usable installation or configuration.

**Independent Test**: Run diagnostics and version reporting, simulate a compatible
update, then simulate update failure and verify configuration and the previous usable
installation remain available.

### Tests for User Story 6

- [ ] T053 [P] [US6] Add CLI contract tests for doctor check names, exit codes, version output, and Traditional Chinese diagnostics in `backend/tests/contract/doctor_version.rs`
- [ ] T054 [P] [US6] Add update integration tests for compatible release success, configuration preservation, interrupted download, and rollback behavior in `backend/tests/integration/update_recovery.rs`

### Implementation for User Story 6

- [X] T055 [US6] Implement login, Sheet, network, configuration, cache, and dashboard health checks in `backend/src/diagnostics/checks.rs` and `backend/src/cli/doctor.rs`
- [ ] T056 [US6] Implement compatible-release discovery, download verification, staged replacement, configuration preservation, and rollback in `backend/src/update/service.rs` and `backend/src/update/recovery.rs`
- [X] T057 [US6] Implement `glucose-dashboard update` and `glucose-dashboard version` command output and exit behavior in `backend/src/cli/update.rs` and `backend/src/cli/version.rs`
- [ ] T058 [US6] Add update status, diagnostic detail, and recovery guidance to the Traditional Chinese CLI and local-service error responses in `backend/src/diagnostics/report.rs` and `backend/src/errors.rs`
- [ ] T059 [US6] Add browser and command-line smoke coverage for doctor, version, update success, and update failure recovery in `tests/e2e/maintenance.spec.ts` and `backend/tests/integration/maintenance.rs`

**Checkpoint**: Users can diagnose common setup problems and update safely without
losing configuration or the last usable installation.

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Verify the integrated product against constitutional quality gates and the
complete quickstart before release.

- [ ] T060 [P] Audit keyboard navigation, semantic labels, focus states, contrast, and color-independent status cues in `frontend/src/components/`, `frontend/src/styles/accessibility.css`, and `frontend/tests/accessibility/dashboard_a11y.test.tsx`
- [ ] T061 [P] Measure startup, filter, refresh, and memory targets using repeatable fixtures in `tests/e2e/performance.spec.ts` and `backend/tests/integration/performance.rs`
- [ ] T062 [P] Review credential handling, logs, error payloads, local configuration permissions, and dependency security in `backend/src/auth/`, `backend/src/config/`, `backend/src/observability.rs`, and `SECURITY.md`
- [ ] T063 Run every scenario in `specs/001-glucose-dashboard/quickstart.md` and record any deviations in `specs/001-glucose-dashboard/quickstart-results.md`
- [ ] T064 Update user-facing installation, setup, command, troubleshooting, and release documentation in `README.md` and `docs/`

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; T001–T005 initialize independent project areas.
- **Foundational (Phase 2)**: Depends on Setup; blocks all user-story implementation.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP increment.
- **User Story 2 (Phase 4)**: Depends on Foundational; its end-to-end flow uses US1
  configuration, but analysis/UI work can begin with fixtures after the foundation.
- **User Story 3 (Phase 5)**: Depends on Foundational and integrates with the US2
  dashboard views for complete synchronized updates.
- **User Story 4 (Phase 6)**: Depends on the US2 record table and US3 selection state.
- **User Story 5 (Phase 7)**: Depends on US1 configuration/startup and can be developed
  in parallel with US2/US3 after the foundation.
- **User Story 6 (Phase 8)**: Depends on US1 configuration and US5 daily startup for
  safe update-preservation validation.
- **Polish (Phase 9)**: Depends on all desired user stories being complete.

### User Story Dependencies

```text
Setup -> Foundational
          ├──> US1 (MVP) ───> US5 ───> US6
          ├──> US2 ──────────┐
          └──> US3 <─────────┘
                    └──> US4
All completed stories -> Polish
```

US2 and the core of US3 can proceed in parallel after Foundational when fixture data is
used. US4 requires the table and selection contracts from US2/US3. US5 and US6 remain
dependent on safe configuration and startup behavior.

### Within Each User Story

- Contract or integration tests are written before their implementation tasks and must
  fail for the missing behavior first.
- Domain models and state types precede services; services precede routes and UI wiring.
- Story-specific browser tests run after the story implementation but before its
  checkpoint is considered complete.

## Parallel Execution Examples

### Setup and Foundational

```text
T001 backend package      || T002 frontend package      || T003 repository tooling
T004 test harness         || T005 installer scaffold
T006 domain types         || T010 config model          || T011 secure credential adapter
T012 errors/logging       || T015 fixture corpus
```

T007–T009 and T013–T014 begin after the types or package boundaries they consume are
available.

### User Story 1

```text
T016 CLI contract tests   || T017 first-run integration tests
T018 OAuth flow           || T023 setup UI
```

T019–T022 integrate the shared configuration and service boundary before T024 runs.

### User Story 2

```text
T025 analysis tests       || T026 frontend component tests
T028 summary cards        || T029 trend chart
```

T027 and T030–T031 integrate the response and page shell; T032 validates the complete
story.

### User Story 3

```text
T033 contract tests        || T034 synchronization integration tests
T035 selection service     || T037 sidebar controls
```

T036 and T038 join source synchronization and frontend state before T039–T040.

### User Story 4

```text
T041 export contract tests || T042 table-control tests
T043 backend projection    || T044 frontend controls
```

T045 finalizes the export encoding before T046 browser validation.

### User Story 5

```text
T047 startup integration tests || T048 repeated-start browser tests
T049 startup orchestration     || T050 single-instance/browser reuse
```

T051–T052 complete the user-visible daily workflow.

### User Story 6

```text
T053 doctor/version tests || T054 update recovery tests
T055 diagnostics         || T056 update service
```

T057–T059 complete CLI output and end-to-end maintenance validation.

## Implementation Strategy

### MVP First (User Story 1)

1. Complete Phase 1 Setup.
2. Complete Phase 2 Foundational, including fixtures and secure configuration.
3. Complete Phase 3 User Story 1.
4. Stop and validate installation, first-run authentication, Sheet validation, startup,
   and dashboard opening using the US1 independent test.
5. Demo or release the MVP slice only after the Constitution quality gates pass.

### Incremental Delivery

1. Add US2 for the read-only dashboard summary and visual hierarchy.
2. Add US3 for filtering, refresh, context-specific calculations, and stale-data safety.
3. Add US4 for record search, sorting, and CSV export.
4. Add US5 for reliable daily reuse and single-instance startup.
5. Add US6 for diagnostics, version reporting, updates, and recovery.
6. Run Phase 9 before each release and keep each completed story independently testable.

## Notes

- `[P]` means the task touches a separate file set and has no dependency on incomplete
  work in its phase.
- `[US1]` through `[US6]` map directly to the six user stories in `spec.md`.
- Every task includes at least one concrete file path and is ordered by implementation
  dependency.
- `tasks.md` intentionally does not create or prescribe a permanent application database.
