use std::sync::Arc;

use gem_evm::{
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use primitives::{Asset, AssetId, Chain};
use serde::{Deserialize, Deserializer, Serialize};

use alloy_core::{
    hex::decode as HexDecode,
    primitives::{Address, AddressError, U256},
    sol_types::SolCall,
};

use crate::{
    network::{jsonrpc_call, AlienHttpMethod, AlienProvider, AlienTarget},
    swapper::SwapperError,
};

const MAYAN_PROGRAM_ID: &str = "FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf";
pub const MAYAN_FORWARDER_CONTRACT: &str = "0x0654874eb7F59C6f5b39931FC45dC45337c967c3";
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

fn deserialize_string_or_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, Unexpected};
    let value: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = value {
        s.parse::<u64>()
            .map(Some)
            .map_err(|_| Error::invalid_value(Unexpected::Str(&s), &"a valid u64"))
    // Convert string to u64
    } else {
        Ok(None)
    }
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
pub struct Token {
    pub name: String,
    pub standard: String,
    pub symbol: String,
    pub mint: String,
    pub verified: bool, // Added
    pub contract: String,
    #[serde(rename = "wrappedAddress")]
    pub wrapped_address: Option<String>, // Added
    #[serde(rename = "chainId")]
    pub chain_id: Option<u64>,
    #[serde(rename = "wChainId")]
    pub w_chain_id: Option<u64>,
    pub decimals: u8,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    #[serde(rename = "coingeckoId")]
    pub coingecko_id: String,
    #[serde(rename = "realOriginChainId")]
    pub real_origin_chain_id: Option<u64>,
    #[serde(rename = "realOriginContractAddress")]
    pub real_origin_contract_address: Option<String>,
    #[serde(rename = "supportsPermit")]
    pub supports_permit: bool,
    #[serde(rename = "hasAuction")]
    pub has_auction: bool, // Added
}

#[derive(Debug, PartialEq)]
pub enum QuoteType {
    SWIFT,
    MCTP,
    SWAP,
    WH,
}

impl ToString for QuoteType {
    fn to_string(&self) -> String {
        match self {
            QuoteType::SWIFT => "SWIFT".to_string(),
            QuoteType::MCTP => "MCTP".to_string(),
            QuoteType::SWAP => "SWAP".to_string(),
            QuoteType::WH => "WH".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "effectiveAmountIn")]
    pub effective_amount_in: f64,
    #[serde(rename = "expectedAmountOut")]
    pub expected_amount_out: f64,
    #[serde(rename = "priceImpact")]
    pub price_impact: Option<f64>,
    #[serde(rename = "minAmountOut")]
    pub min_amount_out: f64,

    #[serde(rename = "minMiddleAmount")]
    pub min_middle_amount: Option<f64>,

    #[serde(rename = "evmSwapRouterAddress")]
    pub evm_swap_router_address: Option<String>,

    #[serde(rename = "evmSwapRouterCalldata")]
    pub evm_swap_router_calldata: Option<String>,
    #[serde(rename = "minReceived")]
    pub min_received: f64,
    #[serde(rename = "gasDrop")]
    pub gas_drop: f64,
    pub price: f64,
    #[serde(rename = "swapRelayerFee")]
    pub swap_relayer_feed: Option<f64>,
    #[serde(rename = "redeemRelayerFee")]
    pub redeem_relayer_fee: Option<f64>,
    #[serde(rename = "refundRelayerFee")]
    pub refund_relayer_fee: Option<f64>,
    #[serde(rename = "solanaRelayerFee")]
    pub solana_relayer_fee: Option<f64>,
    #[serde(rename = "refundRelayerFee64")]
    pub refund_relayer_fee64: String,
    #[serde(rename = "cancelRelayerFee64")]
    pub cancel_relayer_fee64: String,
    #[serde(rename = "submitRelayerFee64")]
    pub submit_relayer_fee64: String,
    #[serde(rename = "solanaRelayerFee64")]
    pub solana_relayer_fee64: Option<String>,
    #[serde(rename = "clientRelayerFeeSuccess")]
    pub client_relayer_fee_success: Option<f64>,
    #[serde(rename = "clientRelayerFeeRefund")]
    pub client_relayer_fee_refund: Option<f64>,
    pub eta: u64,
    #[serde(rename = "etaSeconds")]
    pub eta_seconds: u64,
    #[serde(rename = "clientEta")]
    pub client_eta: String,
    #[serde(rename = "fromToken")]
    pub from_token: Token,
    #[serde(rename = "toToken")]
    pub to_token: Token,
    #[serde(rename = "fromChain")]
    pub from_chain: String,
    #[serde(rename = "toChain")]
    pub to_chain: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u32,
    #[serde(rename = "bridgeFee")]
    pub bridge_fee: f64,
    #[serde(rename = "suggestedPriorityFee")]
    pub suggested_priority_fee: f64,
    #[serde(rename = "onlyBridging")]
    pub only_bridging: bool,
    #[serde(rename = "deadline64")]
    pub deadline64: String,
    #[serde(rename = "referrerBps")]
    pub referrer_bps: Option<u32>,
    #[serde(rename = "protocolBps")]
    pub protocol_bps: Option<u32>,
    #[serde(rename = "swiftMayanContract")]
    pub swift_mayan_contract: Option<String>,
    #[serde(rename = "swiftAuctionMode")]
    pub swift_auction_mode: Option<u8>,
    #[serde(rename = "swiftInputContract")]
    pub swift_input_contract: String,
    #[serde(rename = "swiftInputDecimals")]
    pub swift_input_decimals: u8,
    pub gasless: bool,
    pub relayer: String,
    #[serde(rename = "sendTransactionCost")]
    pub send_transaction_cost: f64,
    #[serde(rename = "maxUserGasDrop")]
    pub max_user_gas_drop: f64,

    #[serde(rename = "rentCost", deserialize_with = "deserialize_string_or_u64")]
    pub rent_cost: Option<u64>,
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
    #[error("Route not found")]
    RouteNotFound,
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
    use std::{sync::Arc, time::Duration};

    use crate::network::mock::AlienProviderMock;

    use super::*;

    #[test]
    fn test_quote_deserialization() {
        let json_data = r#"
        {
      "maxUserGasDrop": 0.01,
      "sendTransactionCost": 0,
      "rentCost": "40000000",
      "gasless": false,
      "swiftAuctionMode": 2,
      "swiftMayanContract": "0xC38e4e6A15593f908255214653d3D947CA1c2338",
      "minMiddleAmount": null,
      "swiftInputContract": "0x0000000000000000000000000000000000000000",
      "swiftInputDecimals": 18,
      "slippageBps": 100,
      "effectiveAmountIn": 0.1,
      "expectedAmountOut": 0.09972757016582551,
      "price": 0.9996900030000002,
      "priceImpact": null,
      "minAmountOut": 0.09873029446416727,
      "minReceived": 0.09873029446416727,
      "route": null,
      "swapRelayerFee": null,
      "swapRelayerFee64": null,
      "redeemRelayerFee": null,
      "redeemRelayerFee64": null,
      "solanaRelayerFee": null,
      "solanaRelayerFee64": null,
      "refundRelayerFee": null,
      "refundRelayerFee64": "1056",
      "cancelRelayerFee64": "25",
      "submitRelayerFee64": "0",
      "deadline64": "1732777812",
      "clientRelayerFeeSuccess": null,
      "clientRelayerFeeRefund": 0.038947098479114796,
      "fromToken": {
        "name": "ETH",
        "standard": "native",
        "symbol": "ETH",
        "mint": "",
        "verified": true,
        "contract": "0x0000000000000000000000000000000000000000",
        "wrappedAddress": "0x4200000000000000000000000000000000000006",
        "chainId": 8453,
        "wChainId": 30,
        "decimals": 18,
        "logoURI": "https://statics.mayan.finance/eth.png",
        "coingeckoId": "weth",
        "pythUsdPriceId": "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        "realOriginContractAddress": "0x4200000000000000000000000000000000000006",
        "realOriginChainId": 30,
        "supportsPermit": false,
        "hasAuction": true
      },
      "fromChain": "base",
      "toToken": {
        "name": "ETH",
        "standard": "native",
        "symbol": "ETH",
        "mint": "8M6d63oL7dvMZ1gNbgGe3h8afMSWJEKEhtPTFM2u8h3c",
        "verified": true,
        "contract": "0x0000000000000000000000000000000000000000",
        "wrappedAddress": "0x4200000000000000000000000000000000000006",
        "chainId": 10,
        "wChainId": 24,
        "decimals": 18,
        "logoURI": "https://statics.mayan.finance/eth.png",
        "coingeckoId": "weth",
        "pythUsdPriceId": "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        "realOriginContractAddress": "0x4200000000000000000000000000000000000006",
        "realOriginChainId": 24,
        "supportsPermit": false,
        "hasAuction": true
      },
      "toTokenPrice": 3602.87682508,
      "toChain": "optimism",
      "mintDecimals": null,
      "gasDrop": 0,
      "eta": 1,
      "etaSeconds": 12,
      "clientEta": "12s",
      "bridgeFee": 0,
      "suggestedPriorityFee": 0,
      "type": "SWIFT",
      "priceStat": {
        "ratio": 0.9996907516582549,
        "status": "GOOD"
      },
      "referrerBps": 0,
      "protocolBps": 3,
      "onlyBridging": false,
      "sourceSwapExpense": 0,
      "relayer": "7dm9am6Qx7cH64RB99Mzf7ZsLbEfmXM7ihXXCvMiT2X1",
      "meta": {
        "advertisedDescription": "Cheapest and Fastest",
        "advertisedTitle": "Best",
        "icon": "https://cdn.mayan.finance/fast_icon.png",
        "switchText": "Switch to the best route",
        "title": "Best"
      }
    }
        "#;

        let quote: Quote = serde_json::from_str(json_data).expect("Failed to deserialize Quote");
        assert_eq!(quote.r#type, "SWIFT");
        assert!(quote.price_impact.is_none());
        assert_eq!(quote.swift_input_decimals, 18);
    }

    #[test]
    fn test_token_deserialization() {
        let json_data = r#"
    {
        "name": "ETH",
        "standard": "native",
        "symbol": "ETH",
        "mint": "",
        "verified": true,
        "contract": "0x0000000000000000000000000000000000000000",
        "wrappedAddress": "0x4200000000000000000000000000000000000006",
        "chainId": 8453,
        "wChainId": 30,
        "decimals": 18,
        "logoURI": "https://statics.mayan.finance/eth.png",
        "coingeckoId": "weth",
        "realOriginChainId": 30,
        "realOriginContractAddress": "0x4200000000000000000000000000000000000006",
        "supportsPermit": false,
        "hasAuction": true
    }"#;

        let token: Token = serde_json::from_str(json_data).expect("Failed to deserialize Token");
        assert_eq!(token.name, "ETH");
        assert!(token.verified);
        assert_eq!(token.chain_id, Some(8453));
        assert_eq!(token.has_auction, true);
        assert_eq!(token.wrapped_address.unwrap(), "0x4200000000000000000000000000000000000006");
    }

    #[test]
    fn test_quote_response_deserialization() {
        let json_data = r#"
    {
        "quotes": [],
        "minimumSdkVersion": [7, 0, 0]
    }"#;

        let response: QuoteResponse = serde_json::from_str(json_data).expect("Failed to deserialize QuoteResponse");
        assert_eq!(response.minimum_sdk_version, vec![7, 0, 0]);
    }
}
