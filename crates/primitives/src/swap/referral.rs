use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ReferralAddress {
    pub evm: String,
    pub solana: String,
    pub sui: String,
    pub ton: String,
    pub tron: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ReferralInfo {
    pub address: ReferralAddress,
    pub bps: u32,
}
