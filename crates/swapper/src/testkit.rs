use crate::{
    FetchQuoteData, ProviderData, ProviderType, Route, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData, SwapperSlippage,
    SwapperSlippageMode,
};
use async_trait::async_trait;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_TOKEN_ID};

use super::{Options, Quote, QuoteRequest};

impl ProviderData {
    pub fn mock() -> Self {
        ProviderData {
            provider: ProviderType::new(SwapperProvider::Okx),
            routes: vec![],
            slippage_bps: 50,
        }
    }
}

impl Route {
    pub fn mock(input: AssetId, output: AssetId) -> Self {
        Route {
            input,
            output,
            route_data: serde_json::json!({
                "fee_tier": "100",
                "min_amount_out": "1",
            })
            .to_string(),
        }
    }
}

impl Options {
    pub fn mock_exact(bps: u32) -> Self {
        Self::new_with_slippage(SwapperSlippage::mock_exact(bps))
    }
}

impl QuoteRequest {
    pub fn mock(chain: Chain, token_id: Option<&str>) -> Self {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(chain, token_id.map(|s| s.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(chain)),
            wallet_address: "address".to_string(),
            destination_address: "address".to_string(),
            value: "1000000".to_string(),
            options: Options::default(),
        }
    }
}

impl Quote {
    pub fn mock(chain: Chain, token_id: Option<&str>) -> Self {
        Quote {
            from_value: "1000000".to_string(),
            min_from_value: None,
            to_value: "1000000".to_string(),
            data: ProviderData::mock(),
            request: QuoteRequest::mock(chain, token_id),
            eta_in_seconds: None,
        }
    }

    pub fn mock_with_provider(provider: SwapperProvider, to_value: &str) -> Self {
        Quote {
            from_value: "1000000".to_string(),
            min_from_value: None,
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: ProviderType::new(provider),
                routes: vec![],
                slippage_bps: 50,
            },
            request: QuoteRequest::mock(Chain::Ethereum, None),
            eta_in_seconds: None,
        }
    }
}

pub fn mock_quote(from_asset: SwapperQuoteAsset, to_asset: SwapperQuoteAsset) -> QuoteRequest {
    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        value: "1000000".into(),
        options: Options {
            slippage: SwapperSlippage {
                mode: SwapperSlippageMode::Auto,
                bps: 50,
            },
            use_max_amount: false,
        },
    }
}

pub fn mock_bitcoin_max_quote(to_asset: SwapperQuoteAsset) -> QuoteRequest {
    let mut request = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)), to_asset);
    request.wallet_address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into();
    request.destination_address = "11111111111111111111111111111111".into();
    request.value = "89100".into();
    request.options.use_max_amount = true;
    request
}

pub fn mock_ton(wallet_address: String) -> QuoteRequest {
    QuoteRequest {
        from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
        to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
        wallet_address: wallet_address.clone(),
        destination_address: wallet_address,
        value: "1000000000".to_string(),
        options: Options {
            slippage: 100.into(),
            use_max_amount: false,
        },
    }
}

type MockResponse = fn() -> Result<Quote, SwapperError>;

#[derive(Debug)]
pub struct MockSwapper {
    provider: ProviderType,
    supported_assets: Vec<SwapperChainAsset>,
    response: MockResponse,
}

impl MockSwapper {
    pub fn new(provider: SwapperProvider, response: MockResponse) -> Self {
        Self {
            provider: ProviderType::new(provider),
            supported_assets: vec![SwapperChainAsset::All(Chain::Ethereum)],
            response,
        }
    }
}

#[async_trait]
impl Swapper for MockSwapper {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.supported_assets.clone()
    }

    async fn get_quote(&self, _request: &QuoteRequest) -> Result<Quote, SwapperError> {
        (self.response)()
    }

    async fn get_quote_data(&self, _quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        todo!("MockSwapper fetch_quote_data not implemented")
    }
}
