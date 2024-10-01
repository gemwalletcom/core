use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TransactionInput {
    pub address: String, // Coinbase / OP_Return will be filtered
    pub value: String,
}
