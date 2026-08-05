# Local Service Contract

## Purpose

The local service is the only boundary between the browser UI and backend capabilities.
The browser must not access Google Sheet credentials, source rows, or backend internals
directly.

## Data exchange conventions

- JSON is used for structured requests and responses.
- Date/time values use an explicit timezone-aware representation at the service boundary.
- Glucose values are represented in mg/dL.
- Errors use a stable machine-readable code plus a Traditional Chinese message.
- Authentication secrets are never included in responses.

## Operations

| Operation | Purpose | Required inputs | Result |
|---|---|---|---|
| `GET /api/health` | Check local service readiness | None | Service status and version |
| `GET /api/config/status` | Read non-sensitive setup state | None | Configured/unconfigured and credential-store status |
| `POST /api/configure` | Complete or replace first-run setup | Sheet metadata and auth completion | Saved configuration status |
| `POST /api/sync` | Read and validate the configured Sheet | None | Sync state, issue list, and current valid records |
| `GET /api/dashboard` | Calculate the active dashboard view | Period, event filter | Summary, trend points, table records, sync metadata |
| `GET /api/records/export.csv` | Export visible table rows | Selection, search, sort | CSV with visible columns and rows |
| `GET /api/diagnostics` | Run required system checks | None | Individual check results and overall status |

The exact wire schema may be implemented as typed request/response structures, but the
operation meanings and error behavior are stable contract requirements.

## Synchronization failure contract

When `POST /api/sync` fails, the response includes:

- `status: "failed"`;
- a stable error code;
- a Traditional Chinese user-facing message;
- `last_successful_sync_at`, if known;
- data payloads marked empty so the browser clears cards, chart points, and table rows.

The service must not return the previous records as current data after a failed sync.

## Analysis response contract

`GET /api/dashboard` returns one coherent selection result containing:

- the normalized selection;
- summary values and context-sensitive percentages;
- trend points with classification labels;
- table rows in source-column order;
- current synchronization metadata;
- any non-blocking data-quality diagnostics.

The browser must render these sections from the same response or an equivalent atomic
state update so that cards, chart, and table cannot drift apart after a filter change.
