use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub from_token_address: String,
    pub to_token_address: String,
    pub from_token_amount: String,
    pub to_wallet_address: String,
    pub slippage_percentage: String,
    pub integrator_fee_percentage: String,
    pub integrator_fee_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub decimals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteEntry {
    pub to_token_amount: String,
    #[serde(rename = "txData")]
    pub transaction_data: TransactionData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionData {
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub to_token: Token,
    pub quotes: Vec<QuoteEntry>,
}
