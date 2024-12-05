use std::{fmt::Display, sync::Arc};

use primitives::Chain;
use serde::{Deserialize, Serialize};

use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget};

use super::constants::{MAYAN_FORWARDER_CONTRACT, MAYAN_PROGRAM_ID};

const SDK_VERSION: &str = "9_7_0";

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    msg: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteParams {
    pub amount: f64,
    pub from_token: String,
    pub from_chain: Chain,
    pub to_token: String,
    pub to_chain: Chain,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_drop: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteOptions {
    #[serde(default = "default_true")]
    pub swift: bool,
    #[serde(default = "default_true")]
    pub mctp: bool,
    #[serde(default = "default_false")]
    pub gasless: bool,
    #[serde(default = "default_false")]
    pub only_direct: bool,
}

impl Default for QuoteOptions {
    fn default() -> Self {
        Self {
            swift: true,
            mctp: true,
            gasless: false,
            only_direct: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Token {
    pub name: String,
    pub standard: String,
    pub symbol: String,
    pub mint: String,
    pub verified: bool,
    pub contract: String,
    pub wrapped_address: Option<String>,
    pub chain_id: Option<u64>,
    pub w_chain_id: Option<u64>,
    pub decimals: u8,

    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub coingecko_id: String,
    pub real_origin_chain_id: Option<u64>,
    pub real_origin_contract_address: Option<String>,
    pub supports_permit: bool,
    pub has_auction: bool,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum QuoteType {
    Swift,
    Mctp, // TODO: do we want to support all types?
    Swap,
    WH,
}

impl Display for QuoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteType::Swift => write!(f, "SWIFT"),
            QuoteType::Mctp => write!(f, "MCTP"),
            QuoteType::Swap => write!(f, "SWAP"),
            QuoteType::WH => write!(f, "WH"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Quote {
    #[serde(rename = "type")]
    pub r#type: String,
    pub effective_amount_in: f64,
    pub expected_amount_out: f64,
    pub price_impact: Option<f64>,
    pub min_amount_out: f64,
    pub min_middle_amount: Option<f64>,
    pub evm_swap_router_address: Option<String>,
    pub evm_swap_router_calldata: Option<String>,
    pub min_received: f64,
    pub gas_drop: f64,
    pub price: f64,
    pub swap_relayer_feed: Option<f64>,
    pub redeem_relayer_fee: Option<f64>,
    pub refund_relayer_fee: Option<f64>,
    pub solana_relayer_fee: Option<f64>,
    pub refund_relayer_fee64: String,
    pub cancel_relayer_fee64: String,
    pub from_token: Token,
    pub to_token: Token,
    pub from_chain: String,
    pub to_chain: String,
    pub slippage_bps: u32,
    pub bridge_fee: f64,
    pub suggested_priority_fee: f64,
    pub only_bridging: bool,
    pub deadline64: String,
    pub referrer_bps: Option<u32>,
    pub protocol_bps: Option<u32>,
    pub swift_mayan_contract: Option<String>,
    pub swift_auction_mode: Option<u8>,
    pub swift_input_contract: String,
    pub swift_input_decimals: u8,
    pub gasless: bool,
    pub relayer: String,
    pub send_transaction_cost: f64,
    pub max_user_gas_drop: f64,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    quotes: Vec<Quote>,

    #[serde(rename = "minimumSdkVersion")]
    pub minimum_sdk_version: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum MayanRelayerError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("SDK version not supported")]
    SdkVersionNotSupported,
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
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
        Self::new("https://price-api.mayan.finance/v3".to_string(), provider)
    }

    pub async fn get_quote(&self, params: QuoteParams, options: Option<QuoteOptions>) -> Result<Vec<Quote>, MayanRelayerError> {
        let options = options.unwrap_or_default();
        let from_chain = if params.from_chain == Chain::SmartChain {
            "bsc".to_string()
        } else {
            params.from_chain.to_string()
        };

        let mut query_params = vec![
            ("swift", options.swift.to_string()),
            ("mctp", options.mctp.to_string()),
            ("gasless", options.gasless.to_string()),
            ("onlyDirect", options.only_direct.to_string()),
            ("solanaProgram", MAYAN_PROGRAM_ID.to_string()),
            ("forwarderAddress", MAYAN_FORWARDER_CONTRACT.to_string()),
            ("amountIn", params.amount.to_string()),
            ("fromToken", params.from_token),
            ("fromChain", from_chain),
            ("toToken", params.to_token),
            ("toChain", params.to_chain.to_string()),
            // ("slippageBps", params.slippage_bps.map_or("auto".to_string(), |v| v.to_string())),
            // ("gasDrop", params.gas_drop.unwrap_or(0).to_string()),
            ("sdkVersion", "9_7_0".to_string()),
        ];

        if let Some(slippage) = params.slippage_bps {
            query_params.push(("slippageBps", slippage.to_string()));
        }
        if let Some(gas_drop) = params.gas_drop {
            query_params.push(("gasDrop", gas_drop.to_string()));
        }
        if let Some(referrer) = params.referrer {
            query_params.push(("referrer", referrer));
        }
        if let Some(referrer_bps) = params.referrer_bps {
            query_params.push(("referrerBps", referrer_bps.to_string()));
        }

        let query = serde_urlencoded::to_string(&query_params).map_err(|e| MayanRelayerError::InvalidParameters(e.to_string()))?;

        let url = format!("{}/quote?{}", self.url, query);

        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| MayanRelayerError::NetworkError(err.to_string()))?;

        let quote_response = serde_json::from_slice::<QuoteResponse>(&data);
        match quote_response {
            Ok(response) => {
                if !self.check_sdk_version(response.minimum_sdk_version) {
                    return Err(MayanRelayerError::SdkVersionNotSupported);
                }

                Ok(response.quotes)
            }
            Err(err) => {
                if let Ok(api_error) = serde_json::from_slice::<ApiError>(&data) {
                    return Err(MayanRelayerError::InvalidResponse(api_error.msg));
                }
                Err(MayanRelayerError::NetworkError(err.to_string()))
            }
        }
    }

    fn check_sdk_version(&self, minimum_version: Vec<u8>) -> bool {
        let sdk_version = SDK_VERSION.split('_').filter_map(|x| x.parse::<u8>().ok()).collect::<Vec<_>>();

        // Major version check
        if sdk_version[0] < minimum_version[0] {
            return false;
        }
        if sdk_version[0] > minimum_version[0] {
            return true;
        }

        // Minor version check
        if sdk_version[1] < minimum_version[1] {
            return false;
        }
        if sdk_version[1] > minimum_version[1] {
            return true;
        }

        if sdk_version[2] >= minimum_version[2] {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_deserialization() {
        let data = include_str!("tests/quote_response.json");
        let quote: Quote = serde_json::from_str(data).expect("Failed to deserialize Quote");
        assert_eq!(quote.r#type, "SWIFT");
        assert_eq!(quote.swift_input_decimals, 18);
    }

    #[test]
    fn test_token_deserialization() {
        let data = include_str!("tests/quote_token_response.json");

        let token: Token = serde_json::from_str(data).expect("Failed to deserialize Token");
        assert_eq!(token.name, "ETH");
        assert!(token.verified);
        assert_eq!(token.chain_id, Some(8453));
        assert_eq!(token.wrapped_address.unwrap(), "0x4200000000000000000000000000000000000006");
    }

    #[test]
    fn test_quote_response_deserialization() {
        let json_data = r#"{
            "quotes": [],
            "minimumSdkVersion": [7, 0, 0]
        }"#;

        let response: QuoteResponse = serde_json::from_str(json_data).expect("Failed to deserialize QuoteResponse");
        assert_eq!(response.minimum_sdk_version, vec![7, 0, 0]);
    }
}
