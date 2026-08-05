# Research: Local Glucose Dashboard

## Decision 1: Use the constitution's two-layer application boundary

**Decision**: Implement a Rust local service and CLI under `backend/`, and a React
browser UI under `frontend/`. The browser communicates only through documented local
service contracts.

**Rationale**: This directly satisfies the Constitution's required flow and keeps Google
Sheet access and business rules out of the presentation layer. It also allows backend
and frontend tests to run independently.

**Alternatives considered**:

- A single frontend-only application was rejected because it would expose source access
  and analysis rules to the browser and violate the required domain boundary.
- Electron or Tauri was rejected because the Constitution explicitly excludes them
  unless a future ADR proves the browser model insufficient.

## Decision 2: Use desktop OAuth with OS-protected credential storage

**Decision**: First-run configuration opens the provider's browser sign-in flow for a
desktop/local application, receives the authorization result through the local app flow,
and stores refresh credentials only in the supported operating system's secure
credential store. The non-sensitive configuration file stores only a credential
reference and Sheet metadata.

**Rationale**: This preserves the one-time setup experience while preventing secrets from
being written to plain-text files. If the secure store is unavailable, the product must
require re-authentication rather than silently weakening storage protection.

**Alternatives considered**:

- Re-authentication on every startup was rejected because it breaks the zero-maintenance
  daily workflow.
- Plain-text or application-managed unencrypted credentials were rejected because they
  create avoidable local disclosure risk.
- A hosted authentication service was rejected because the product is local-first.

## Decision 3: Treat the Google Sheet schema as a versioned public contract

**Decision**: Require the exact five headers and preserve source row order and values in
  a validated `GlucoseRecord` representation. Rows with missing fields, unsupported
  dates, invalid glucose ranges, or unknown events are excluded from analysis and
  reported with diagnostics.

**Rationale**: The Sheet is the only source of truth, so schema drift must be visible and
  deterministic. Valid rows must remain usable even when other rows are malformed.

**Alternatives considered**:

- Header aliases or automatic column guessing were rejected because they can silently
  reinterpret user data.
- Persisting a normalized database copy was rejected because it would create a second
  source of truth.

## Decision 4: Apply context-sensitive glucose standards per record

**Decision**: Classification uses the record's event context: fasting after 8 hours is
  70–99 mg/dL, pre-meal after at least 4 hours is 70–100 mg/dL, and post-meal or bedtime
  uses the 2-hour rule where values at or above 140 mg/dL are high. Aggregate summaries
  classify each record with its own rule before calculating percentages.

**Rationale**: This is the user's explicit clarification and avoids treating fasting,
  pre-meal, post-meal, and bedtime readings as interchangeable measurements. It also
  resolves the mismatch between the supplied image and the earlier universal 70–180 rule.

**Alternatives considered**:

- A universal 70–180 mg/dL rule was rejected by the clarification.
- A single chart threshold for all events was rejected because it would misclassify
  context-specific readings.

## Decision 5: Clear analysis views after synchronization failure

**Decision**: On a failed startup or manual synchronization, clear summary values, chart
  points, and table rows. Show a Traditional Chinese error state and the last successful
  synchronization time when available.

**Rationale**: The user selected this behavior to prevent stale data from appearing
  current. The synchronization state remains available as metadata, but stale records are
  not presented as an analysis result.

**Alternatives considered**:

- Retaining the last successful records with a stale badge was rejected by clarification.
- Showing a blank page without the last successful synchronization time was rejected
  because it weakens diagnosis and recovery.

## Decision 6: Use the supplied image as the MVP UI contract

**Decision**: Preserve the image's desktop composition: header, left sidebar, three
summary cards, trend panel, and record table. Keep the MVP in Traditional Chinese,
retain standard technical abbreviations, and use light high-contrast surfaces with
blue/green/yellow-red semantics.

**Rationale**: The image is the most concrete visual requirement and makes component
boundaries, placement, and information priority testable without prescribing exact
pixels or icon assets.

**Alternatives considered**:

- A generic responsive dashboard was rejected because it would leave the requested
  hierarchy and control placement ambiguous.
- Multilingual UI in the MVP was deferred to keep the first release scope bounded.

## Decision 7: Test contracts and user journeys at multiple levels

**Decision**: Use backend unit tests for parsing and analysis, backend integration tests
for Sheet and local-service contracts, frontend component/integration tests for all
view-state transitions, and browser smoke tests for the quickstart journeys.

**Rationale**: The product has multiple failure boundaries: authentication, source
  parsing, classification, synchronization, and UI rendering. Testing only the UI or
  only the backend would miss contract regressions.

**Alternatives considered**:

- Manual-only validation was rejected because the Constitution requires independently
  verifiable, regression-resistant changes.
- End-to-end-only testing was rejected because malformed-row and classification rules
  require fast deterministic unit coverage.
