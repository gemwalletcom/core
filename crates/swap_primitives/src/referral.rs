use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ReferralAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sui: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ton: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tron: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ReferralInfo {
    pub address: ReferralAddress,
    pub bps: u32,
}
