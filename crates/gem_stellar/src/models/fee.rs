use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarFees {
    pub last_ledger_base_fee: String,
    pub fee_charged: StellarFeeCharged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarFeeCharged {
    pub min: String,
    pub p95: String,
}
