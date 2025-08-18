use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Serialize, Deserialize)]
pub struct SolanaBalance {
    pub value: UInt64,
}

pub struct SolanaBalanceValue {
    pub amount: String,
}
