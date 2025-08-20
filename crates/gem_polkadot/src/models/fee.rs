use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotEstimateFee {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub partial_fee: u64,
}
