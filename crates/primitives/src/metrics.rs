use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedError {
    pub message: String,
    pub count: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParserStatus {
    pub errors: Vec<ReportedError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsumerStatus {
    pub total_processed: u64,
    pub total_errors: u64,
    pub last_success: Option<u64>,
    pub last_result: Option<String>,
    pub avg_duration: u64,
    pub errors: Vec<ReportedError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobStatus {
    pub interval: u64,
    pub duration: u64,
    pub last_success: Option<u64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<u64>,
    pub error_count: u64,
}
