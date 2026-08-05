# Google Sheet Contract

## Required header

The first row MUST contain these exact headers, in this exact order, with no added spaces:

```text
血糖量測日期時間
事件
量測血糖值(mg/dl)
備註1
備註2
```

Header mismatch is a blocking source-contract error. The product must identify the
problem in Traditional Chinese and must not calculate partial results from an unknown
schema.

## Accepted row values

- Date/time: `yyyy/MM/dd HH:mm` or `yyyy-MM-dd HH:mm`.
- Event: `空腹血糖`, `午餐前`, `午餐後`, `晚餐前`, `晚餐後`, or `睡前`.
- Glucose: integer from 20 through 600 mg/dL, inclusive.
- Remarks: optional text, preserved without reinterpretation.

## Invalid-row behavior

| Problem | Result |
|---|---|
| Missing date/time, event, or glucose | Exclude row; record warning |
| Unsupported date format | Exclude row; record error |
| Glucose outside 20–600 | Exclude row; record invalid-data error |
| Unknown event | Label `Unknown Event`; exclude from statistics; retain for diagnostics |
| Valid row among invalid rows | Include valid row in analysis |

The product does not rewrite the user's Sheet. It reads the source, validates rows, and
maps valid values to `GlucoseRecord`.

## Context mapping

| Event | Classification context |
|---|---|
| `空腹血糖` | Fasting after 8 hours: 70–99 mg/dL |
| `午餐前`, `晚餐前` | Pre-meal after at least 4 hours: 70–100 mg/dL |
| `午餐後`, `晚餐後` | Post-meal at 2 hours: >=140 mg/dL is high |
| `睡前` | Post-meal 2-hour rule: >=140 mg/dL is high |

## Source-of-truth rule

Every analysis run begins from the current validated Sheet data. A local cache may
reduce repeated reads during a single active session, but it must not become an
independent permanent record store or override a successful fresh read.
