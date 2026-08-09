# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A **local-first** glucose dashboard for non-developer Windows users. No server, no database — the user points the app at a Google Sheet, the Rust backend reads it on demand, computes analysis, and serves a React UI. The browser is the only UI surface. Per the project Constitution (`doc/Glucose Dashboard 專案憲法（Constitution）.md`): Google Sheet is the single source of truth; the system must never build its own database or persist Sheet data beyond ephemeral analysis.

The UI and all user-facing messages are **Traditional Chinese** (zh-TW). Technical command names and field codes remain English. Match this convention in any new user-facing strings.

## Spec-Driven Workflow (IMPORTANT)

This repo uses **Spec Kit**. Before writing any code, read `Agents.md` and the relevant spec docs in order:

1. `doc/PRD.md` → 2. `specs/001-glucose-dashboard/spec.md` → 3. `plan.md` → 4. `data-model.md` → 5. `contracts/*.md` → 6. `tasks.md`

Rules from `Agents.md`:
- **One task at a time** from `tasks.md`. Do not start the next task without Product Owner review.
- Never modify `spec.md` or `plan.md` — raise an issue instead.
- Every task requires unit tests, a full test run, and zero warnings.
- After each task: update docs if affected, propose a git commit message in **Traditional Chinese**, and **do not auto-push**.
- Stop and wait for review when the current task is done.

The `.agents/skills/speckit-*` and `.specify/` directories back this workflow; `specs/001-glucose-dashboard/contracts/` (cli, dashboard-ui, google-sheet, local-service) are the binding interface contracts.

## Build, Run, Test

All commands use the Makefile (manifest lives at `backend/Cargo.toml`):

```bash
make run            # Build frontend, apply config, then cargo run (starts backend on 127.0.0.1:3000)
make build          # Build frontend + backend
make frontend-build # npm install + npm run build in frontend/
make backend-build  # cargo build
make configure      # cargo run -- config <SHEET_ID> <FIXTURE>  (SHEET_ID=demo, FIXTURE=backend/tests/fixtures/valid-sheet.csv by default)
make test           # cargo test (backend) + frontend build (type-check via tsc)
make format         # cargo fmt --all
make clean
```

Running the backend alone: `cargo run --manifest-path backend/Cargo.toml` serves the API and the built frontend from `frontend/dist` (via `ServeDir` fallback in `api/router.rs`). The dev frontend (`npm --prefix frontend run dev`, Vite on :5173) proxies `/api` to `:3000`.

Single Rust test: `cargo test --manifest-path backend/Cargo.toml <test_name>` (e.g. `cargo test --manifest-path backend/Cargo.toml calculates_summary`).

Frontend tests: `npm --prefix frontend run test` (vitest). Frontend type-check is part of `npm run build` (`tsc -b`).

Formatting/lint: Rust via `cargo fmt` (`rustfmt.toml`: `max_width = 100`, edition 2021). Frontend via `frontend/eslint.config.js`. Indentation: 2 spaces (JS/TS), 4 spaces (Rust) per `.editorconfig`.

## Architecture

### Data flow (request time, no persistence)
```
Google Sheet (CSV export) → SyncService.load() → sheet_parser → GlucoseRecord[] + DataQualityIssue[]
   → analysis::selection::filter() → analysis::summary::calculate() → JSON → React
```
Records are re-fetched and re-parsed on **every** `/api/dashboard` and `/api/sync` call. The only thing persisted is `.glucose-dashboard.json` (config + `last_successful_sync_at`).

### Backend (`backend/src/`)
- **`main.rs`** — entrypoint + CLI dispatch. Subcommands: `version`, `doctor`, `config <sheet_id> [fixture]`, `update`. No subcommand → starts the axum server on `127.0.0.1:3000`.
- **`api/`** — axum router (`router.rs` builds `Router` with `ApiState { config }`). Routes: `/api/health`, `/api/config/status`, `/api/configure`, `/api/config/test-connection`, `/api/sync`, `/api/dashboard`, `/api/records/export.csv`, `/api/diagnostics`. `ServeDir` fallback serves `frontend/dist`. CORS is permissive (local-only app).
- **`ingestion/`** — `sync_service.rs` (`SyncService`) fetches the Sheet CSV via `reqwest` (public export URL, no OAuth token used for the public-read path) or reads a local fixture; `sheet_parser.rs` validates headers and rows. **Header names and order are immutable** (see Google Sheet Contract) — a mismatch is a blocking error that stops analysis.
- **`domain/`** — `records.rs` (`GlucoseRecord`, `Event`, `Classification`, `Period`, `AnalysisSelection`, `AnalysisSummary`) and `data_quality.rs` (`DataQualityIssue`, `IssueCode`, `IssueSeverity`). `GlucoseRecord::classify()` applies **context-specific** thresholds (fasting 70–99, pre-meal 70–100, post-meal/bedtime 70–139 / ≥140 high) — for an "All" selection the summary aggregates each record's own classification, never one global threshold.
- **`analysis/`** — `selection.rs` (filter by period/event/search; search affects table only, never summary), `summary.rs` (averages, min/max, estimated HbA1c via `(avg+46.7)/28.7`, in-range/low/high percentages), `classification.rs` (delegates to `GlucoseRecord::classify()`).
- **`config/`** — `store.rs` (`ConfigStore`, JSON file at `GLUCOSE_CONFIG_PATH` or `.glucose-dashboard.json`), `model.rs` (`LocalConfiguration`), `service.rs` (`configure()` + `normalize_sheet_reference()` which parses full Sheet URLs and extracts id/gid).
- **`errors.rs`** — `AppError` (thiserror) → JSON `{status, code, message}` with HTTP status mapping (`NotConfigured`→428, `Sync`→502, `Invalid`→400, `Internal`→500).
- **`auth/`** — `credential_store.rs` is a stub (Windows-only / `GLUCOSE_ALLOW_INSECURE_DEV_AUTH` env); `oauth.rs` present. OAuth is not wired into the fetch path.
- **`diagnostics/`** — `doctor` checks (login, Sheet, network, config, cache, dashboard).
- **`runtime/`**, **`update/`** — startup/browser-launch and self-update scaffolding (not yet connected to a release source).

### Frontend (`frontend/src/`)
React + Vite + TypeScript. Components under `components/{layout,records,summary,trend}/`. `app/App.tsx` is the top-level page (period/event/search filters, config form, connection test, sync). `state/dashboard_store.ts` (`useDashboard` hook) calls `services/local_service.ts` (thin `fetch` wrappers over the REST API). `types/index.ts` mirrors the backend's serialized shapes.

### Invalid-row handling (key contract)
Rows with missing required fields, unparseable dates, out-of-range glucose (20–600), or unknown events are **excluded from statistics** but reported as `DataQualityIssue`s with row numbers. Valid rows among invalid ones remain usable. The app never rewrites the user's Sheet.

## Key Files & Constants

- Sheet headers (immutable order): `backend/src/ingestion/sheet_parser.rs::HEADERS` — `血糖量測日期時間`, `事件`, `量測血糖值(mg/dl)`, `備註1`, `備註2`.
- Event names (zh-TW, exact): `空腹血糖`, `午餐前`, `午餐後`, `晚餐前`, `晚餐後`, `睡前` — mapped in `domain/records.rs::Event::parse`.
- eAG/HbA1c formula centralized in `analysis/summary.rs` (intentionally labeled an estimate; change in one place if clinically approved formula changes).
- Test fixture: `backend/tests/fixtures/valid-sheet.csv` (also `tests/fixtures/sample-sheet.csv`).

## Notes

- The `cli.sh` file at repo root configures alternative model/proxy aliases for Claude Code itself — it is **not** part of the application; ignore it for product work.
- Installer target: `%LOCALAPPDATA%\GlucoseDashboard` on Windows (see `installer/README.md`); must not overwrite the config file on update.
- `make test` runs backend `cargo test` plus a frontend **build** (type-check). Backend unit tests live in `backend/tests/unit/`; contract/integration harness dirs exist under `backend/tests/contract/` and `backend/tests/integration/`.