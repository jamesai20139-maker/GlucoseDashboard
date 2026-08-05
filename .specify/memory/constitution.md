<!--
Sync Impact Report
- Version change: unversioned template -> 1.0.0
- Modified principles: placeholder principles -> I. Local-First Product,
  II. Google Sheet as the Source of Truth, III. Layered Architecture and Domain
  Boundaries, IV. Maintainable and Testable Engineering, V. Usability, Performance,
  and Safe Evolution
- Added sections: Product and Technology Constraints; Development Workflow and
  Quality Gates
- Removed sections: none; template placeholders were replaced with project-specific
  governance
- Follow-up TODOs: The original ratification date is not recorded and must be supplied
  as TODO(RATIFICATION_DATE).
-->
# Glucose Dashboard Constitution

## Core Principles

### I. Local-First Product

The core product MUST run locally. Except for the Google Sheets API and the minimum
network access required for installation or updates, the Dashboard MUST NOT require a
cloud service, hosted server, or user-managed database to perform its primary analysis.
The product MUST preserve the goals of zero setup, zero maintenance, one-command
installation, one-command startup, and automatic updating where supported by the target
environment. This keeps the tool usable by people who do not operate infrastructure.

### II. Google Sheet as the Source of Truth

Google Sheet MUST remain the only authoritative source of glucose records. The system
MUST NOT introduce a permanent application database or persist a synchronized copy of
the Sheet as a second source of truth. The Sheet header names, order, and meanings are
a public data contract and MUST NOT change without an explicit, versioned migration
decision. Data fetched from the Sheet MUST be converted into a domain model such as
`BloodGlucoseRecord` before analysis; analysis code MUST NOT depend directly on raw
Sheet rows. This protects data integrity while allowing users to continue managing
their records in the tool they already use.

### III. Layered Architecture and Domain Boundaries

The system MUST retain the following primary flow:

```text
Google Sheet -> Google Sheets API -> Rust Backend -> REST API -> React Dashboard -> Browser
```

The Rust backend MUST own data access, domain modelling, analysis, caching, and API
contracts. The React frontend MUST own presentation and user interaction. The frontend
MUST communicate with the backend only through the REST API and MUST NOT access backend
internals or the Google Sheet directly. Changes that replace this architecture or
materially increase coupling MUST be documented in an Architecture Decision Record
(ADR) and approved before implementation. These boundaries keep business logic
testable and allow the UI and analysis engine to evolve independently.

### IV. Maintainable and Testable Engineering

Every feature MUST be modular, maintainable, reusable where justified, and independently
verifiable. New behavior MUST include tests at the lowest practical level and integration
tests for changed API, data, or cross-layer contracts. Changes MUST prefer extending or
correcting existing modules over introducing duplicate functionality. Large refactors
MUST NOT be bundled with ordinary feature work; they require an ADR describing the
problem, alternatives, migration risk, and rollback strategy. This limits change scope
and gives both human and AI contributors a reliable feedback loop.

### V. Usability, Performance, and Safe Evolution

The Dashboard MUST follow an information-first design: important summary information
MUST be visible on the initial view, and changing a filter MUST update the summary cards,
trend chart, and detail table consistently. The interface MUST favor high readability,
fewest practical interactions, consistent components, and immediate feedback.

Performance is a product requirement. Under the normal supported usage scenario, the
Dashboard SHOULD start within three seconds and the backend SHOULD use less than
100 MB of memory; any regression against an established baseline MUST be measured and
treated as a defect or explicitly accepted with rationale. Installation and updates MUST
preserve user configuration and MUST NOT require reconfiguring the Google Sheet unless a
documented breaking migration is unavoidable. These rules protect trust and make future
growth compatible with the existing product.

## Product and Technology Constraints

The product is a local glucose analysis and visualization tool, not a medical system,
electronic health-record system, or diagnostic service. Product behavior and copy MUST
not represent dashboard output as medical advice or a clinical diagnosis.

The reference technology stack is Rust with Cargo, Axum, and Tokio for the backend, and
React with TypeScript and Vite for the frontend. The default execution environment is a
system browser. Electron and Tauri MUST NOT be introduced unless an ADR records why the
browser-based architecture cannot satisfy a demonstrated requirement.

The CLI MUST provide concise, consistent, memorable, and diagnosable commands for
installation, startup, update, configuration, system checks, and version reporting. A
normal user MUST be able to complete the supported installation and startup flow without
writing application code or operating a separate server.

## Development Workflow and Quality Gates

All requirements, specifications, plans, tasks, and implementations MUST comply with
this Constitution. Before implementation, a feature MUST have a written requirement and
an implementation plan proportional to its risk. API, Google Sheet header, domain model,
or architecture changes MUST identify compatibility and migration impact.

Before a change is considered complete, contributors MUST verify the affected behavior,
run relevant automated tests, and confirm that existing functionality remains intact.
User-visible changes MUST include appropriate documentation or release notes. A change
that cannot meet a quality gate MUST record the exception, its risk, and a follow-up
owner or task before merge.

When requirements conflict, decisions MUST follow this order unless an approved ADR
documents the exception: (1) simple and usable operation, (2) data integrity, (3)
existing architecture, (4) maintainability, (5) user experience, (6) performance, and
(7) future extensibility.

## Governance

This Constitution is the highest-level development guidance for Glucose Dashboard.
PRDs, specifications, plans, tasks, implementation details, and review decisions MUST
comply with it. A conflict MUST be resolved by changing the conflicting proposal or by
submitting an ADR; contributors MUST NOT silently bypass a principle.

Amendments MUST be proposed as a documented change to this file, identify affected
principles and dependent artifacts, explain compatibility and migration impact, and be
reviewed by the project owner before adoption. The amendment MUST update the Sync Impact
Report and `Last Amended` date. If an amendment changes architecture, data contracts,
or user configuration behavior, the related ADR and migration plan MUST be completed
before implementation of the dependent change.

Constitution versions use Semantic Versioning. A MAJOR increment is required for a
backward-incompatible removal or redefinition of a principle. A MINOR increment is
required for a new principle or a materially expanded governance requirement. A PATCH
increment is required for clarifications, wording, typo fixes, and other non-semantic
refinements. Every feature review MUST check compliance with the current version, and
the project owner MUST review this Constitution whenever a release changes an API,
data contract, architecture boundary, installation/update flow, or quality gate.

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): original adoption date not recorded | **Last Amended**: 2026-08-05
