use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarFees {
    pub min: u64,
    pub last_ledger_base_fee: u64,
    pub fee_charged: StellarFeeCharged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarFeeCharged {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub min: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub p95: u64,
}
