use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonInfo {
    pub jetton_content: JettonContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonContent {
    pub data: JettonInfoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonInfoMetadata {
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub decimals: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonBalances {
    pub balances: Vec<JettonBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonBalance {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
    pub jetton: Jetton,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jetton {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleJettonBalance {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWalletsResponse {
    pub jetton_wallets: Vec<JettonWallet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWallet {
    pub address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
    pub jetton: String,
}
