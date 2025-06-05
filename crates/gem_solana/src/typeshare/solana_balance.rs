use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::Int;

#[typeshare(swift = "Sendable")]
#[derive(Serialize, Deserialize)]
pub struct SolanaBalance {
    pub value: Int,
}

#[typeshare(swift = "Sendable")]
pub struct SolanaBalanceValue {
    pub amount: String,
}
