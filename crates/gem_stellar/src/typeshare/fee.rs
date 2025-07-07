use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StellarFees {
    pub last_ledger_base_fee: String,
    pub fee_charged: StellarFeeCharged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StellarFeeCharged {
    pub min: String,
    pub p95: String,
}
