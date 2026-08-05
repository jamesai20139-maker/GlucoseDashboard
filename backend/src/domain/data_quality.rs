use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum IssueCode {
    MissingField,
    InvalidDate,
    InvalidGlucose,
    UnknownEvent,
    HeaderMismatch,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataQualityIssue {
    pub source_row_number: usize,
    pub severity: IssueSeverity,
    pub code: IssueCode,
    pub message_zh_tw: String,
}
