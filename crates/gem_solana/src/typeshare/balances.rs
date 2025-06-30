use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::typeshare::UInt64;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
pub struct SolanaBalance {
    pub value: UInt64,
}

#[typeshare(swift = "Sendable")]
pub struct SolanaBalanceValue {
    pub amount: String,
}
