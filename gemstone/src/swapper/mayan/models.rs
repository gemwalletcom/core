use crate::swapper::mayan::{MAYAN_FORWARDER_CONTRACT, MAYAN_PROGRAM_ID};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_optional_u64_from_str, deserialize_u64_from_str, serialize_optional_u64, serialize_u64};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize)]
pub struct QuoteOptions {
    pub swift: bool,
    pub fast_mctp: bool,
    pub only_direct: bool,
}

impl Default for QuoteOptions {
    fn default() -> Self {
        Self {
            swift: true,
            fast_mctp: false,
            only_direct: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteUrlParams {
    pub swift: bool,
    pub fast_mctp: bool,
    pub only_direct: bool,
    pub solana_program: String,
    pub forwarder_address: String,
    pub amount_in64: String,
    pub from_token: String,
    pub from_chain: String,
    pub to_token: String,
    pub to_chain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_bps: Option<u32>,
    pub sdk_version: String,
}

impl QuoteUrlParams {
    pub fn new(
        amount_in64: String,
        from_token: String,
        from_chain: Chain,
        to_token: String,
        to_chain: Chain,
        options: &QuoteOptions,
        slippage_bps: Option<String>,
        referrer: Option<String>,
        referrer_bps: Option<u32>,
    ) -> Self {
        let from_chain_str = if from_chain == Chain::SmartChain {
            "bsc".to_string()
        } else if from_chain == Chain::AvalancheC {
            "avalanche".to_string()
        } else {
            from_chain.to_string()
        };

        Self {
            swift: options.swift,
            fast_mctp: options.fast_mctp,
            only_direct: options.only_direct,
            solana_program: MAYAN_PROGRAM_ID.to_string(),
            forwarder_address: MAYAN_FORWARDER_CONTRACT.to_string(),
            amount_in64,
            from_token,
            from_chain: from_chain_str,
            to_token,
            to_chain: to_chain.to_string(),
            slippage_bps,
            referrer,
            referrer_bps,
            sdk_version: "10_4_0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub verified: bool,
    pub contract: String,
    pub wrapped_address: Option<String>,
    pub w_chain_id: u64,
    pub decimals: u8,
}

#[derive(Debug, PartialEq)]
pub enum QuoteType {
    Swift,
    FastMctp,
}

impl Display for QuoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteType::Swift => write!(f, "SWIFT"),
            QuoteType::FastMctp => write!(f, "FAST_MCTP"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub effective_amount_in64: u64,
    pub expected_amount_out: f64,
    pub min_amount_out: f64,
    pub min_middle_amount: Option<f64>,
    pub evm_swap_router_address: Option<String>,
    pub evm_swap_router_calldata: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_u64_from_str", serialize_with = "serialize_optional_u64")]
    pub refund_relayer_fee64: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64_from_str", serialize_with = "serialize_optional_u64")]
    pub cancel_relayer_fee64: Option<u64>,
    pub from_token: Token,
    pub to_token: Token,
    pub from_chain: String,
    pub to_chain: String,
    pub slippage_bps: u32,
    pub deadline64: String,
    pub referrer_bps: Option<u32>,
    pub protocol_bps: Option<u32>,
    pub swift_mayan_contract: Option<String>,
    pub swift_input_contract: Option<String>,
    pub swift_auction_mode: Option<u8>,
    pub swift_input_decimals: u8,
    pub relayer: String,
}

#[derive(Debug, Deserialize)]
pub struct QuoteError {
    pub code: String,
    pub msg: String,
}

#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    pub quotes: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum QuoteResult {
    Success(QuoteResponse),
    Error(QuoteError),
}
