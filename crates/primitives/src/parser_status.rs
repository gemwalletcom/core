use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParserStatus {
    pub errors: Vec<ParserError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserError {
    pub message: String,
    pub count: u64,
    pub timestamp: u64,
}
