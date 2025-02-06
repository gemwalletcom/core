use crate::{
    network::AlienProvider,
    swapper::{FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapperError},
};
use async_trait::async_trait;
use gem_evm::uniswap::deployment;
use primitives::Chain;
use std::sync::Arc;

#[derive(Debug)]
pub struct UniswapV4 {}

impl UniswapV4 {
    fn support_chain(&self, chain: &Chain) -> bool {
        deployment::v4::get_uniswap_router_deployment_by_chain(chain).is_some()
    }
}

#[async_trait]
impl GemSwapProvider for UniswapV4 {
    fn provider(&self) -> SwapProvider {
        SwapProvider::UniswapV4
    }
    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapChainAsset::All(*x)).collect()
    }
    async fn fetch_quote(&self, _request: &SwapQuoteRequest, _provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        todo!()
    }
    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}
