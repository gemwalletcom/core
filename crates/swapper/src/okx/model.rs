use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct OkxClientConfig {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub project: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OkxApiResponse<T> {
    pub code: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenInfo {
    pub token_contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteData {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub to_token_amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TransactionData {
    #[serde(default)]
    pub data: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub gas: String,
    #[serde(default)]
    pub signature_data: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteParams {
    pub chain_index: String,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub slippage_percent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex_ids: Option<&'static str>,
    pub fee_percent: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SwapParams {
    pub chain_index: String,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub user_wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_transaction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_slippage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auto_slippage_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex_ids: Option<&'static str>,
    pub fee_percent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_token_referrer_wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_token_referrer_wallet_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SwapDataResult {
    pub tx: TransactionData,
}
