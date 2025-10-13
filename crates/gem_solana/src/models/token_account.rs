use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaTokenInfo {
    pub token_amount: SolanaTokenAmount,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaTokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

#[derive(Serialize, Deserialize)]
pub struct TokenAccountBalanceValue {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

#[derive(Serialize, Deserialize)]
pub struct TokenAccountBalance {
    pub value: TokenAccountBalanceValue,
}
