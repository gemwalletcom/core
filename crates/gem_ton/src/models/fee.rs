use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateFee {
    pub address: String,
    pub body: String,
    pub ignore_chksig: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fees {
    pub source_fees: Fee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub in_fwd_fee: i32,
    pub storage_fee: i32,
}
