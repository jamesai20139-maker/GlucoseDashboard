use crate::domain::{AnalysisSelection, GlucoseRecord};

pub fn filter(records: &[GlucoseRecord], selection: &AnalysisSelection) -> Vec<GlucoseRecord> {
    records
        .iter()
        .filter(|record| selection.period.contains(record.measured_at))
        .filter(|record| {
            selection
                .event
                .as_ref()
                .map(|event| event == &record.event)
                .unwrap_or(true)
        })
        .filter(|record| {
            selection
                .search
                .as_ref()
                .map(|text| {
                    let needle = text.to_lowercase();
                    record.event.label_zh_tw().to_lowercase().contains(&needle)
                        || record.glucose_mg_dl.to_string().contains(&needle)
                        || record.remark_1.to_lowercase().contains(&needle)
                        || record.remark_2.to_lowercase().contains(&needle)
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect()
}
