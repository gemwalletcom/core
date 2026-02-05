use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobStatus {
    pub interval: u64,
    pub duration: u64,
    pub last_success: Option<u64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<u64>,
    pub error_count: u64,
}
