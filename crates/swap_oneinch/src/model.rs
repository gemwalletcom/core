use std::{collections::HashMap, str::FromStr};

use gem_evm::address::EthereumAddress;
use primitives::SwapQuoteData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub src: String,
    pub dst: String,
    pub from: String,
    pub amount: String,
    pub slippage: f64,
    pub disable_estimate: bool,
    pub fee: f64,
    pub referrer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub to_amount: String,
    pub tx: Option<SwapResultTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub status_code: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SwapResponse {
    Success(SwapResult),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResultTransaction {
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas: i64,
}

impl SwapResultTransaction {
    pub fn get_data(&self) -> SwapQuoteData {
        SwapQuoteData {
            to: EthereumAddress::from_str(&self.to.clone())
                .unwrap()
                .to_checksum(),
            value: self.value.clone(),
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: i64,
}

pub type Tokenlist = HashMap<String, Token>;
