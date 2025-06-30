use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonEstimateFee {
    pub address: String,
    pub body: String,
    pub ignore_chksig: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonFees {
    pub source_fees: TonFee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonFee {
    pub in_fwd_fee: i32,
    pub storage_fee: i32,
}