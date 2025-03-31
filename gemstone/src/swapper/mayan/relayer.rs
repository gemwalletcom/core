use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

use super::{MAYAN_FORWARDER_CONTRACT, MAYAN_PROGRAM_ID};
use crate::network::{AlienProvider, AlienTarget};
use crate::swapper::SwapperError;
use alloy_primitives::Address;
use gem_evm::serializer::{deserialize_address, deserialize_optional_address, serialize_address, serialize_optional_address};
use primitives::Chain;
use serde_serializers::{deserialize_optional_u64_from_str, deserialize_u64_from_str, serialize_optional_u64, serialize_u64};

pub const MAYAN_API_URL: &str = "https://price-api.mayan.finance/v3";

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
    pub fn from_params(
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub verified: bool,
    pub contract: String,
    pub wrapped_address: Option<String>,
    pub chain_id: Option<u64>,
    pub w_chain_id: u64,
    pub decimals: u8,
    pub real_origin_chain_id: Option<u64>,
    pub real_origin_contract_address: Option<String>,
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
    pub min_received: f64,
    pub gas_drop: f64,
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
    #[serde(deserialize_with = "deserialize_optional_address", serialize_with = "serialize_optional_address")]
    pub swift_mayan_contract: Option<Address>,
    pub swift_auction_mode: Option<u8>,
    #[serde(deserialize_with = "deserialize_address", serialize_with = "serialize_address")]
    pub swift_input_contract: Address,
    pub swift_input_decimals: u8,
    pub relayer: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum QuoteResult {
    Success(QuoteResponse),
    Error(QuoteError),
}

#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    pub quotes: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteError {
    pub code: String,
    pub msg: String,
}

#[derive(Debug)]
pub struct MayanRelayer {
    url: String,
    provider: Arc<dyn AlienProvider>,
}

impl MayanRelayer {
    pub fn new(url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { url, provider }
    }

    pub fn default_relayer(provider: Arc<dyn AlienProvider>) -> Self {
        Self::new(MAYAN_API_URL.to_string(), provider)
    }

    pub async fn get_quote(&self, params: QuoteUrlParams) -> Result<Vec<Quote>, SwapperError> {
        let query = serde_urlencoded::to_string(&params).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        let url = format!("{}/quote?{}", self.url, query);
        let target = AlienTarget::get(&url);

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let result = serde_json::from_slice::<QuoteResult>(&data).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        match result {
            QuoteResult::Success(response) => Ok(response.quotes),
            QuoteResult::Error(error) => match error.code.as_str() {
                "QUOTE_NOT_FOUND" => Err(SwapperError::NoQuoteAvailable),
                _ => Err(SwapperError::ComputeQuoteError { msg: error.msg }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_url_params_from_params() {
        let options = QuoteOptions::default();
        let params = QuoteUrlParams::from_params(
            "100".to_string(),
            "ETH".to_string(),
            Chain::Ethereum,
            "USDC".to_string(),
            Chain::Solana,
            &options,
            Some("50".to_string()),
            Some("referrer".to_string()),
            Some(10),
        );

        assert_eq!(params.amount_in64, "100");
        assert_eq!(params.from_token, "ETH");
        assert_eq!(params.from_chain, "ethereum");
        assert_eq!(params.to_token, "USDC");
        assert_eq!(params.to_chain, "solana");
        assert_eq!(params.solana_program, MAYAN_PROGRAM_ID.to_string());
        assert_eq!(params.forwarder_address, MAYAN_FORWARDER_CONTRACT.to_string());
        assert_eq!(params.slippage_bps, Some("50".to_string()));
        assert_eq!(params.referrer, Some("referrer".to_string()));
        assert_eq!(params.referrer_bps, Some(10));
        assert!(params.swift);
        assert!(!params.fast_mctp);
        assert!(!params.only_direct);
    }

    #[test]
    fn test_smart_chain_conversion() {
        let options = QuoteOptions::default();
        let params = QuoteUrlParams::from_params(
            "100".to_string(),
            "BNB".to_string(),
            Chain::SmartChain,
            "USDC".to_string(),
            Chain::Solana,
            &options,
            None,
            None,
            None,
        );

        assert_eq!(params.from_chain, "bsc");
    }

    #[test]
    fn test_quote_deserialization() {
        let data = include_str!("test/quote_response.json");
        let result = serde_json::from_str::<QuoteResponse>(data).unwrap();
        let quote = result.quotes.first().unwrap();

        assert_eq!(quote.r#type, "SWIFT");
        assert_eq!(quote.swift_input_decimals, 18);
    }

    #[test]
    fn test_token_deserialization() {
        let data = include_str!("test/quote_token_response.json");

        let token: Token = serde_json::from_str(data).expect("Failed to deserialize Token");
        assert_eq!(token.name, "ETH");
        assert!(token.verified);
        assert_eq!(token.chain_id.unwrap(), 8453);
        assert_eq!(token.wrapped_address.unwrap(), "0x4200000000000000000000000000000000000006");
    }
}
