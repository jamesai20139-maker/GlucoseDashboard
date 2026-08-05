# Implementation Plan: Local Glucose Dashboard

**Branch**: `001-glucose-dashboard` | **Date**: 2026-08-05 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-glucose-dashboard/spec.md`

## Summary

Build a local, single-user glucose analysis dashboard for Windows 10 and Windows 11.
The product will use Google Sheet as its authoritative source, validate and normalize
rows into a domain model, calculate context-sensitive glucose summaries, and expose a
browser dashboard through a local Rust service. A small CLI will handle installation
entry points, startup, configuration, diagnostics, updates, and version reporting.

The implementation will keep the backend responsible for source access, validation,
classification, analysis, and contracts. The frontend will render the reference-image
layout and communicate through explicit local contracts. Non-sensitive configuration will
be stored locally, OAuth credentials will use the operating system secure credential
store, and fetched records will remain ephemeral or cacheable without becoming a second
source of truth.

## Technical Context

**Language/Version**: Rust stable with Cargo; TypeScript managed by the frontend package
manifest and locked per release

**Primary Dependencies**: Tokio and Axum for the local service, React and Vite for the
browser UI, a Google Sheets client, OAuth 2 desktop authentication, OS credential-store
integration, JSON serialization, structured logging, and a charting library selected
during implementation

**Storage**: Google Sheet as the authoritative source; OS secure credential store for
OAuth credentials; a local non-sensitive configuration file; in-memory or replaceable
cache only, with no permanent application database

**Testing**: Rust unit and integration tests, frontend component tests, browser smoke
tests, contract fixtures, data-quality fixtures, and the quickstart scenarios in
`quickstart.md`

**Target Platform**: Windows 10 and Windows 11 with the system browser

**Project Type**: Local CLI plus browser dashboard with a Rust service and React
frontend in one repository

**Performance Goals**: Dashboard startup and initial valid-data display within 3 seconds
under normal supported use; filter updates feel immediate and keep all views synchronized;
normal backend memory usage remains below 100 MB

**Constraints**: One local user and one Google Sheet for MVP; no permanent database; no
Electron or Tauri; Traditional Chinese user-facing text; exact Sheet headers and
context-sensitive glucose rules; synchronization failures clear displayed data; updates
preserve non-sensitive configuration and secure credentials

**Scale/Scope**: Personal glucose history for one configured Sheet; MVP excludes
multi-user, multi-Sheet, mobile/tablet, medical diagnosis, AI analysis, PDF reporting,
health-platform integrations, physician sharing, and plugins

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Local-First Product**: PASS. The core workflow runs locally and uses the network
  only for Google Sheet access, authentication, installation, and updates.
- **II. Google Sheet as the Source of Truth**: PASS. No permanent application database
  is planned; all analysis starts from validated Sheet rows.
- **III. Layered Architecture and Domain Boundaries**: PASS. Source access, domain
  analysis, local contracts, and browser presentation remain separate.
- **IV. Maintainable and Testable Engineering**: PASS. The plan includes domain tests,
  contract tests, integration tests, browser checks, and bounded modules.
- **V. Usability, Performance, and Safe Evolution**: PASS. The plan preserves the
  reference layout, 3-second startup target, synchronized filters, and safe updates.
- **Product and Technology Constraints**: PASS. The planned stack is Rust plus React in
  a system browser; Electron and Tauri are not introduced.
- **Governance and Quality Gates**: PASS. The design produces explicit contracts,
  validation fixtures, and a runnable quickstart before implementation tasks are made.

## Project Structure

### Documentation (this feature)

```text
specs/001-glucose-dashboard/
├── plan.md                 # This implementation plan
├── research.md             # Phase 0 decisions and alternatives
├── data-model.md           # Phase 1 domain entities and state transitions
├── quickstart.md           # End-to-end validation guide
├── contracts/
│   ├── cli.md              # CLI commands, output, and exit behavior
│   ├── dashboard-ui.md     # Visual and interaction contract
│   ├── google-sheet.md     # Source Sheet contract
│   └── local-service.md    # Local service and data contracts
└── tasks.md                # Created later by $speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── auth/
│   ├── config/
│   ├── ingestion/
│   ├── domain/
│   ├── analysis/
│   ├── api/
│   ├── cli/
│   ├── diagnostics/
│   └── update/
└── tests/
    ├── contract/
    ├── integration/
    └── fixtures/

frontend/
├── package.json
├── vite.config.*
├── src/
│   ├── app/
│   ├── components/
│   │   ├── layout/
│   │   ├── sidebar/
│   │   ├── summary/
│   │   ├── trend/
│   │   └── records/
│   ├── services/
│   ├── state/
│   ├── types/
│   └── styles/
└── tests/
    ├── components/
    ├── integration/
    └── fixtures/

tests/
├── e2e/
└── fixtures/
```

**Structure Decision**: Use a two-part application boundary: `backend/` owns source
access, domain rules, analysis, CLI, and local service contracts; `frontend/` owns the
browser presentation and interaction state. Shared behavior is specified through the
feature contracts and test fixtures rather than by coupling frontend code to backend
internals.

## Complexity Tracking

No Constitution violations require justification. The two-part structure is explicitly
required by the project's layered architecture principle and does not introduce an
additional product or deployment system.

## Phase 0: Research

Research decisions are recorded in [research.md](./research.md). The research resolves
toolchain, authentication, source ingestion, context classification, failure behavior,
frontend contract, and testing choices without changing the product scope.

## Phase 1: Design and Contracts

The data model is recorded in [data-model.md](./data-model.md). It defines source-row
validation, context-sensitive classification, summary calculations, synchronization
state, and secure local configuration.

The external and user-facing contracts are recorded under [contracts/](./contracts/):

- [cli.md](./contracts/cli.md) defines the supported commands and exit behavior.
- [dashboard-ui.md](./contracts/dashboard-ui.md) defines the reference-image layout,
  Traditional Chinese content, filters, states, and visual semantics.
- [google-sheet.md](./contracts/google-sheet.md) defines the exact source headers,
  accepted formats, event values, and invalid-row behavior.
- [local-service.md](./contracts/local-service.md) defines the local service data
  exchange, synchronization behavior, and error envelope.

The end-to-end validation procedure is recorded in [quickstart.md](./quickstart.md).

## Post-Design Constitution Check

- **Local-first and source-of-truth gates** remain PASS: no permanent data mirror is
  introduced, and the local service is only an access and analysis boundary.
- **Layering gate** remains PASS: the service contracts prevent the frontend from
  reaching Google Sheet or backend internals directly.
- **Testability gate** remains PASS: every critical data rule and failure transition has
  a fixture or quickstart scenario.
- **Usability and performance gates** remain PASS: the design preserves the visual
  hierarchy, synchronized views, Traditional Chinese MVP, and measurable performance
  targets.
- **Safe evolution gate** remains PASS: source headers, CLI commands, local-service
  contracts, and secure configuration behavior are versioned design surfaces.
