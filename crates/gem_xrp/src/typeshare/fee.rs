use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPFee {
    pub drops: XRPDrops,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPDrops {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub minimum_fee: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub median_fee: u64,
}
