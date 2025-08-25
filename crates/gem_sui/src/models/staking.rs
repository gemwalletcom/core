#[cfg(feature = "rpc")]
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
#[cfg(feature = "rpc")]
use serde_serializers::deserialize_bigint_from_str;

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStakeDelegation {
    pub validator_address: String,
    pub staking_pool: String,
    pub stakes: Vec<SuiStake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiSystemState {
    pub epoch: String,
    pub epoch_start_timestamp_ms: String,
    pub epoch_duration_ms: String,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStake {
    pub staked_sui_id: String,
    pub status: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub principal: BigInt,
    pub stake_request_epoch: String,
    pub stake_active_epoch: String,
    #[serde(default, deserialize_with = "deserialize_optional_bigint_from_str")]
    pub estimated_reward: Option<BigInt>,
}

#[cfg(feature = "rpc")]
fn deserialize_optional_bigint_from_str<'de, D>(deserializer: D) -> Result<Option<BigInt>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => Ok(Some(s.parse().map_err(serde::de::Error::custom)?)),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidators {
    pub apys: Vec<SuiValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidator {
    pub address: String,
    pub apy: f64,
}
