use crate::{
    FetchQuoteData, ProviderType, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData, SwapperSlippage,
    SwapperSlippageMode, config::get_swap_config,
};
use async_trait::async_trait;
use primitives::Chain;

use super::{Options, Quote, QuoteRequest, SwapperMode};

pub fn mock_quote(from_asset: SwapperQuoteAsset, to_asset: SwapperQuoteAsset) -> QuoteRequest {
    let config = get_swap_config();

    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        value: "1000000".into(),
        mode: SwapperMode::ExactIn,
        options: Options {
            slippage: SwapperSlippage {
                mode: SwapperSlippageMode::Auto,
                bps: 50,
            },
            fee: Some(config.referral_fee.clone()),
            preferred_providers: vec![],
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

    async fn fetch_quote(&self, _request: &QuoteRequest) -> Result<Quote, SwapperError> {
        (self.response)()
    }

    async fn fetch_quote_data(&self, _quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        todo!("MockSwapper fetch_quote_data not implemented")
    }
}
