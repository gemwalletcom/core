use crate::Asset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPreloadInput {
    pub asset: Asset,
    pub sender_address: String,
    pub destination_address: String,
}
