use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosLedger {
    pub chain_id: i32,
    pub ledger_version: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosCoinInfo {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}
