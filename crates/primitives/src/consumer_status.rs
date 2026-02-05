use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsumerStatus {
    pub total_processed: u64,
    pub total_errors: u64,
    pub last_success: Option<u64>,
    pub last_result: Option<String>,
    pub avg_duration: u64,
    pub errors: Vec<ConsumerError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerError {
    pub message: String,
    pub count: u64,
    pub timestamp: u64,
}
