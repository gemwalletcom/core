use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_u64_from_str, serialize_bigint, serialize_u64};
#[cfg(feature = "rpc")]
use std::{error::Error, str::FromStr};
use sui_transaction_builder::ObjectInput;
use sui_types::{Address, Digest};

#[cfg(feature = "rpc")]
use super::SuiCoin;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinAsset {
    pub coin_object_id: Address,
    pub coin_type: String,
    pub digest: Digest,
    #[serde(deserialize_with = "deserialize_bigint_from_str", serialize_with = "serialize_bigint")]
    pub balance: BigInt,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub version: u64,
}

impl CoinAsset {
    pub fn to_input(&self) -> ObjectInput {
        ObjectInput::owned(self.coin_object_id, self.version, self.digest)
    }
}

#[cfg(feature = "rpc")]
impl TryFrom<SuiCoin> for CoinAsset {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(coin: SuiCoin) -> Result<Self, Self::Error> {
        Ok(Self {
            coin_object_id: Address::from_str(&coin.coin_object_id)?,
            coin_type: coin.coin_type,
            digest: Digest::from_str(&coin.digest)?,
            balance: coin.balance,
            version: coin.version.parse()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinResponse {
    pub data: Vec<CoinAsset>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}
