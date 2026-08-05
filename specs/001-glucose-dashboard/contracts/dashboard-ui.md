# Dashboard UI Contract

## Visual reference

The MVP follows [`Dashboard Image.png`](../../../doc/Dashboard%20Image.png) for layout
hierarchy and control placement. Exact pixels, icon artwork, and typography may vary,
but the information order and interaction relationships are contractual.

## Layout

```text
Header: product identity | theme/settings placeholder | user placeholder
Sidebar: period -> custom dates -> refresh/status -> event filters -> references
Main:    three summary cards -> trend panel -> record table
```

The primary desktop view keeps the summary cards, trend panel, and record table visible
without horizontal scrolling at supported desktop widths. The MVP is light mode with a
high-contrast surface hierarchy.

## Required components

### Header

- Shows the Traditional Chinese product name and the English product subtitle.
- Shows last update information where available.
- Provides manual refresh.
- Theme and user/settings controls may be present as reserved controls, but dark mode,
  multi-user behavior, and advanced settings are outside MVP scope.

### Sidebar

The order is fixed:

1. Day/week/month/quarter controls.
2. Custom start and end date.
3. Immediate refresh button.
4. Connection status and last successful synchronization time.
5. All, fasting, lunch-before, lunch-after, dinner-before, dinner-after, and bedtime
   event filters.
6. Informational reference standards.

### Summary cards

Three cards show average glucose, estimated HbA1c/eAG, and TIR as the percentage within
the applicable context-specific reference range. Values are empty during a sync failure
or when the selection has no valid records.

### Trend panel

The panel includes a title, period selector, time-based line chart, context-sensitive
reference regions, legend, and inspectable points. Abnormal points use a distinct red
visual treatment. Inspecting a point shows date, time, glucose value, event, and remarks.

### Record table

The table panel includes a search control and CSV export control. Columns are ordered as:

1. `血糖量測日期時間`
2. `事件`
3. `量測血糖值(mg/dl)`
4. `備註1`
5. `備註2`

Search and sorting affect only visible row presentation. CSV export contains the rows
currently visible after the active selection and search, with the same columns and
values. Empty search results have a no-results state.

## State contracts

- **Loading**: show a clear Traditional Chinese loading state and do not show partial
  aggregate results.
- **Success**: show synchronized cards, chart, and table from one selected record set.
- **Empty selection**: show empty states and no misleading zero values.
- **Sync failure**: clear cards, chart points, and table rows; show an error and last
  successful synchronization time when available.
- **Credential failure**: explain the sign-in problem and provide a retry path.

## Color semantics

- Blue: primary actions and product interaction.
- Green: within the applicable reference range.
- Yellow/orange: low or warning context.
- Red: high or abnormal context.
- White/light gray: background and card surfaces.

Color is supplementary; labels and values must remain understandable without relying on
color alone.
