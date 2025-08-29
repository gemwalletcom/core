use num_bigint::BigUint;
use primitives::AssetId;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str};

use crate::models::rpc::{Info, Parsed, ValueData, ValueResult};
pub use num_bigint::BigInt;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: i64,
    pub mint: String,
    pub owner: String,
    pub ui_token_amount: TokenAmount,
}

#[derive(Debug, Clone)]
pub struct TokenBalanceChange {
    pub asset_id: AssetId,
    pub amount: BigInt,
}

impl TokenBalance {
    pub fn new(account_index: i64, mint: String, owner: String, ui_token_amount: TokenAmount) -> Self {
        Self {
            account_index,
            mint,
            owner,
            ui_token_amount,
        }
    }

    pub fn get_amount(&self) -> BigUint {
        self.ui_token_amount.amount.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountInfo {
    pub pubkey: String,
    pub account: TokenAccountData,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountData {
    pub data: Parsed<Info<TokenAccountInfoData>>,
    pub owner: String,
    pub lamports: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountInfoData {
    pub mint: Option<String>,
    pub token_amount: Option<TokenAmount>,
    pub stake: Option<StakeInfo>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StakeInfo {
    pub delegation: StakeDelegation,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StakeDelegation {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub activation_epoch: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub deactivation_epoch: u64,
    pub stake: String,
    pub voter: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

impl Default for TokenAmount {
    fn default() -> Self {
        Self { amount: BigUint::from(0u64) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub decimals: i32,
    pub supply: String,
    pub extensions: Option<Vec<Extension>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionBase<T> {
    #[serde(rename = "extension")]
    pub extension_type: String,
    pub state: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Extension {
    TokenMetadata(ExtensionBase<TokenMetadata>),
    Other(ExtensionBase<serde_json::Value>),
}

pub type ResultTokenInfo = ValueResult<ValueData<Parsed<Info<TokenInfo>>>>;

impl ResultTokenInfo {
    pub fn info(&self) -> TokenInfo {
        self.value.data.parsed.info.clone()
    }
}
