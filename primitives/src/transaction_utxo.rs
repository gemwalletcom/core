use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TransactionInput {
    pub addresses: Vec<String>, // Coinbase / OP_Return will be filtered
    pub value: String,
}
