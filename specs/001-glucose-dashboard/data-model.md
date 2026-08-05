# Data Model: Local Glucose Dashboard

## SourceSheetContract

Represents the exact Google Sheet structure used by the MVP.

| Field | Type | Required | Validation |
|---|---|---:|---|
| `血糖量測日期時間` | DateTime text | Yes | `yyyy/MM/dd HH:mm` or `yyyy-MM-dd HH:mm` |
| `事件` | String | Yes | One of the six supported event names |
| `量測血糖值(mg/dl)` | Integer | Yes | Inclusive range 20–600 mg/dL |
| `備註1` | String | No | Preserved as supplied |
| `備註2` | String | No | Preserved as supplied |

The header names and order are immutable for the MVP. A header mismatch prevents
analysis and produces a user-visible Traditional Chinese error.

## GlucoseRecord

Represents one valid source row after parsing. It is the only record type consumed by
analysis.

| Field | Type | Description |
|---|---|---|
| `source_row_number` | Positive integer | Source location for diagnostics and traceability |
| `measured_at` | DateTime | Parsed measurement date and time |
| `event` | Enum | Fasting, lunch-before, lunch-after, dinner-before, dinner-after, bedtime |
| `glucose_mg_dl` | Integer | Validated glucose value from 20 to 600 |
| `remark_1` | Optional string | First user remark |
| `remark_2` | Optional string | Second user remark |
| `classification_context` | Enum | Fasting, pre-meal, post-meal, or bedtime |

Rows with missing required values, unsupported dates, out-of-range glucose, or unknown
events do not become `GlucoseRecord` instances. Duplicate source rows are preserved;
the MVP does not deduplicate user data.

## ClassificationRule

Maps a measurement context to the user's reference standard.

| Context | Reference rule | Low | In reference range | High |
|---|---|---|---|---|
| Fasting after 8 hours | 70–99 mg/dL | `<70` | `70–99` | `>99` |
| Pre-meal after at least 4 hours | 70–100 mg/dL | `<70` | `70–100` | `>100` |
| Post-meal at 2 hours | `>=140 mg/dL` is high | `<70` | `70–139` | `>=140` |
| Bedtime | Uses post-meal 2-hour rule | `<70` | `70–139` | `>=140` |

The classification result is calculated per record. For an All-events selection, the
summary aggregates each record's own classification rather than applying one threshold
to the entire selection.

## AnalysisSelection

Defines the records included in all dashboard views.

| Field | Type | Rules |
|---|---|---|
| `period` | Preset enum or custom range | Day, week, month, quarter, or custom start/end |
| `event_filter` | Enum | All, fasting, pre-meal, post-meal, bedtime |
| `search_text` | Optional string | Applies to visible table fields only; does not alter summary calculations |
| `sort` | Field plus direction | Applies to visible table order only; does not alter summary calculations |

Changing the period or event filter recalculates summary, trend, and table together.
Search and sort change only the table presentation.

## AnalysisSummary

Calculated from valid `GlucoseRecord` instances matching `AnalysisSelection`.

| Field | Type | Description |
|---|---|---|
| `record_count` | Non-negative integer | Number of valid selected records |
| `average_mg_dl` | Optional decimal | Average of selected glucose values; empty for no records |
| `minimum_mg_dl` | Optional integer | Lowest selected value; empty for no records |
| `maximum_mg_dl` | Optional integer | Highest selected value; empty for no records |
| `estimated_hba1c_percent` | Optional decimal | Documented estimate derived from average |
| `estimated_average_glucose_mg_dl` | Optional decimal | eAG paired with the estimate |
| `in_reference_percent` | Optional percentage | Records within their context-specific range |
| `low_percent` | Optional percentage | Records below their context-specific lower bound |
| `high_percent` | Optional percentage | Records above their context-specific upper bound |

When `record_count` is zero, all numeric fields are empty and the UI displays the
specified empty state rather than zeros.

## SynchronizationState

Tracks source loading and the dashboard's data availability.

```text
NotConfigured -> Authenticating -> Loading -> Succeeded
                         |             |
                         +-----------> Failed
Failed ------------------------------> Authenticating (retry)
Succeeded ---------------------------> Loading (manual refresh)
```

| State | Meaning | Dashboard data |
|---|---|---|
| `NotConfigured` | No valid local configuration exists | Setup state |
| `Authenticating` | User sign-in or credential retrieval is in progress | Loading state |
| `Loading` | Sheet read and validation are in progress | Loading state |
| `Succeeded` | Current fetch completed and valid records are available | Render records and summaries |
| `Failed` | Fetch or validation failed | Clear records; show error and last successful sync time |

## LocalConfiguration

Stores non-sensitive information required to restart the product.

| Field | Type | Storage rule |
|---|---|---|
| `sheet_id` | String | Local non-sensitive configuration |
| `sheet_name` | String | Local non-sensitive configuration |
| `credential_reference` | String | Reference only; no secret value |
| `schema_version` | Integer | Supports compatible configuration migration |
| `last_successful_sync_at` | Optional DateTime | Diagnostic metadata |

The OAuth credential itself is stored only in the OS secure credential store. If that
store is unavailable, the configuration remains incomplete and the user must sign in
again when needed.

## DataQualityIssue

Represents a source-row problem that prevents analysis.

| Field | Type | Description |
|---|---|---|
| `source_row_number` | Positive integer | Row where the issue occurred |
| `severity` | Warning or Error | Missing value warning or parse/header error |
| `code` | Enum | MissingField, InvalidDate, InvalidGlucose, UnknownEvent, HeaderMismatch |
| `message_zh_tw` | String | Traditional Chinese user-facing explanation |

Quality issues are retained for diagnostics but do not become analyzable records.
