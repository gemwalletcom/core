use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable")]
pub struct TransactionUtxoInput {
    pub address: String, // Coinbase / OP_Return will be filtered
    pub value: String,
}
