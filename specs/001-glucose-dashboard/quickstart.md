# Quickstart Validation Guide

This guide validates the MVP behavior described in [spec.md](./spec.md), the data rules
in [data-model.md](./data-model.md), and the contracts under [contracts/](./contracts/).
It is a validation guide, not an implementation tutorial.

## Prerequisites

- Windows 10 or Windows 11.
- A Google account with read access to a test Sheet.
- A test Sheet with the exact headers from [google-sheet.md](./contracts/google-sheet.md).
- Rust stable and a current Node.js package toolchain for local development validation.
- A system browser.

## Prepare a validation Sheet

Create a test Sheet with these rows after the exact header row:

| Date/time | Event | Glucose | Remark 1 | Remark 2 |
|---|---|---:|---|---|
| `2026/07/07 06:30` | `空腹血糖` | 88 |  |  |
| `2026/07/07 08:00` | `空腹血糖` | 102 |  |  |
| `2026/07/07 12:30` | `午餐前` | 98 |  |  |
| `2026/07/07 14:30` | `午餐後` | 156 | `餐後2小時` |  |
| `2026/07/07 22:30` | `睡前` | 142 |  |  |
| `2026/07/07 23:00` | `Unknown` | 110 |  |  |
| `not-a-date` | `午餐前` | 100 |  |  |
| `2026/07/07 16:00` | `午餐前` | 700 |  |  |

Expected valid records: the first five rows. The Unknown event, invalid date, and
out-of-range glucose must be excluded from statistics and reported as data-quality
issues.

## Start the planned application

After implementation, use the product commands defined in [cli.md](./contracts/cli.md):

```text
glucose-dashboard config
glucose-dashboard doctor
glucose-dashboard
```

Expected results:

1. `config` opens browser authentication, validates Sheet access, and does not write a
   plain-text OAuth secret.
2. `doctor` reports login, Sheet, network, configuration, cache, and dashboard checks.
3. The default command opens the local dashboard in the system browser.

## Validate the dashboard

1. Confirm the header, left sidebar, three summary cards, trend panel, and record table
   follow [dashboard-ui.md](./contracts/dashboard-ui.md).
2. Confirm the UI is Traditional Chinese except for approved technical abbreviations.
3. Select each period preset and a custom range; confirm cards, chart, and table update
   together.
4. Select All, fasting, pre-meal, post-meal, and bedtime; confirm each record uses its
   context-specific classification rule.
5. Search and sort the table; confirm the summary and chart do not change.
6. Export CSV; confirm it contains exactly the visible rows and columns.

## Validate failure behavior

1. Remove or rename a required Sheet header. Confirm analysis stops and the UI explains
   the exact source-contract error.
2. Revoke access or make the Sheet unavailable. Confirm cards, chart, and table clear;
   confirm the last successful sync time is shown when available.
3. Enter a missing field, invalid date, unknown event, and out-of-range glucose. Confirm
   valid rows remain usable and each invalid row receives the correct issue.
4. Make the OS credential store unavailable in a controlled test environment. Confirm
   the product requests authentication again and never writes a plain-text credential.
5. Simulate an update failure. Confirm the previous installation and non-sensitive
   configuration remain usable.

## Quality gates

- Run backend unit and integration tests.
- Run frontend component and interaction tests.
- Run browser smoke tests for setup, dashboard, filtering, sync failure, and CSV export.
- Verify startup time is within 3 seconds under normal supported use.
- Verify normal backend memory usage remains below 100 MB.
- Review the result against the functional requirements and all acceptance scenarios in
  [spec.md](./spec.md) before creating implementation tasks.
