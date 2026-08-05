use crate::domain::{AnalysisSummary, Classification, GlucoseRecord};

pub fn calculate(records: &[GlucoseRecord]) -> AnalysisSummary {
    if records.is_empty() {
        return AnalysisSummary {
            record_count: 0,
            average_mg_dl: None,
            minimum_mg_dl: None,
            maximum_mg_dl: None,
            estimated_hba1c_percent: None,
            estimated_average_glucose_mg_dl: None,
            in_reference_percent: None,
            low_percent: None,
            high_percent: None,
        };
    }
    let total: i32 = records.iter().map(|record| record.glucose_mg_dl).sum();
    let average = total as f64 / records.len() as f64;
    let low = records
        .iter()
        .filter(|record| record.classify() == Classification::Low)
        .count();
    let high = records
        .iter()
        .filter(|record| record.classify() == Classification::High)
        .count();
    let pct = |count: usize| count as f64 * 100.0 / records.len() as f64;
    // The estimate is intentionally labeled as an estimate; the conversion is kept
    // centralized so a clinically approved formula can be changed in one place.
    let hba1c = (average + 46.7) / 28.7;
    AnalysisSummary {
        record_count: records.len(),
        average_mg_dl: Some((average * 10.0).round() / 10.0),
        minimum_mg_dl: records.iter().map(|record| record.glucose_mg_dl).min(),
        maximum_mg_dl: records.iter().map(|record| record.glucose_mg_dl).max(),
        estimated_hba1c_percent: Some((hba1c * 10.0).round() / 10.0),
        estimated_average_glucose_mg_dl: Some((average * 10.0).round() / 10.0),
        in_reference_percent: Some(pct(records.len() - low - high)),
        low_percent: Some(pct(low)),
        high_percent: Some(pct(high)),
    }
}

#[cfg(test)]
mod tests {
    use super::calculate;
    use crate::domain::{Event, GlucoseRecord};
    use chrono::{DateTime, Utc};

    fn record(value: i32, event: Event) -> GlucoseRecord {
        GlucoseRecord {
            source_row_number: 1,
            measured_at: DateTime::parse_from_rfc3339("2026-07-07T06:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            event,
            glucose_mg_dl: value,
            remark_1: String::new(),
            remark_2: String::new(),
        }
    }

    #[test]
    fn calculates_summary_and_context_percentages() {
        let summary = calculate(&[
            record(88, Event::Fasting),
            record(102, Event::Fasting),
            record(142, Event::Bedtime),
        ]);
        assert_eq!(summary.record_count, 3);
        assert_eq!(summary.minimum_mg_dl, Some(88));
        assert_eq!(summary.maximum_mg_dl, Some(142));
        assert_eq!(summary.low_percent, Some(0.0));
        assert_eq!(summary.high_percent, Some(66.66666666666667));
    }
}
