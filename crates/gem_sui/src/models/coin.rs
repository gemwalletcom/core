use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
use num_bigint::BigInt;
#[cfg(feature = "rpc")]
use serde_serializers::deserialize_bigint_from_str;

#[cfg(feature = "rpc")]
use super::account::Owner;
use super::core::Coin;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinMetadata {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiObject {
    pub object_id: String,
    pub digest: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedCoins<T> {
    pub coin_type: String,
    pub coins: Vec<T>,
    pub address_balance: u64,
}

impl<T> Default for OwnedCoins<T> {
    fn default() -> Self {
        Self {
            coin_type: String::new(),
            coins: Vec::new(),
            address_balance: 0,
        }
    }
}

impl<T> OwnedCoins<T> {
    pub fn new(coin_type: String, coins: Vec<T>, address_balance: u64) -> Self {
        Self {
            coin_type,
            coins,
            address_balance,
        }
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> OwnedCoins<U> {
        OwnedCoins {
            coin_type: self.coin_type,
            coins: self.coins.into_iter().map(f).collect(),
            address_balance: self.address_balance,
        }
    }

    pub fn try_map<U, E>(self, f: impl FnMut(T) -> Result<U, E>) -> Result<OwnedCoins<U>, E> {
        Ok(OwnedCoins {
            coin_type: self.coin_type,
            coins: self.coins.into_iter().map(f).collect::<Result<_, _>>()?,
            address_balance: self.address_balance,
        })
    }
}

impl OwnedCoins<Coin> {
    pub fn coin_total(&self) -> u64 {
        self.coins.iter().map(|coin| coin.balance).fold(0, u64::saturating_add)
    }

    pub fn total(&self) -> u64 {
        self.coin_total().saturating_add(self.address_balance)
    }
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub total_balance: BigInt,
    #[serde(default)]
    /// Amount in the per-address balance accumulator.
    pub address_balance: u64,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub owner: Owner,
    #[serde(rename = "coinType")]
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
}
