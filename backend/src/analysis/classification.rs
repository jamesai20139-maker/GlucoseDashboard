use crate::domain::{Classification, GlucoseRecord};

pub fn classify(record: &GlucoseRecord) -> Classification {
    record.classify()
}
