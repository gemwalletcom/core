use super::{client::CetusClient, models::CetusPool};
use crate::{
    network::AlienProvider,
    swapper::{FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapperError},
};
use async_trait::async_trait;
use gem_sui::SUI_COIN_TYPE_FULL;
use primitives::{AssetId, Chain};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Cetus {}

impl Cetus {}

impl Cetus {
    pub fn get_coin_address(asset_id: &AssetId) -> String {
        if asset_id.is_native() {
            return SUI_COIN_TYPE_FULL.into();
        }
        asset_id.token_id.clone().unwrap()
    }
}

#[async_trait]
impl GemSwapProvider for Cetus {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Cetus
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![SwapChainAsset::All(Chain::Sui)]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = CetusClient::new(provider.clone());
        let from_coin = Cetus::get_coin_address(&request.from_asset);
        let to_coin = Cetus::get_coin_address(&request.to_asset);

        let pools = client
            .get_pool_by_token(&from_coin, &to_coin)
            .await?
            .iter()
            .map(|x| CetusPool::from(x.clone()))
            .collect::<Vec<CetusPool>>();

        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        todo!()
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        Ok(true)
    }
}
