use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPreloadInput {
    pub sender_address: String,
    pub destination_address: String,
}
