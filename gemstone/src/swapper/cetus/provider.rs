use super::{
    client::CetusClient,
    models::{CetusPool, CetusPoolObject, CetusPoolType},
};
use crate::{
    debug_println,
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapperError},
};
use async_trait::async_trait;
use gem_sui::{
    jsonrpc::{ObjectDataOptions, SuiRpc},
    SUI_COIN_TYPE_FULL,
};
use primitives::{AssetId, Chain};
use std::sync::Arc;

#[derive(Debug)]
pub struct Cetus {
    provider: SwapProviderType,
}

impl Default for Cetus {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::Cetus),
        }
    }
}

impl Cetus {
    pub fn boxed() -> Box<dyn GemSwapProvider> {
        Box::new(Self::default())
    }

    pub fn get_coin_address(asset_id: &AssetId) -> String {
        if asset_id.is_native() {
            return SUI_COIN_TYPE_FULL.into();
        }
        asset_id.token_id.clone().unwrap()
    }

    async fn fetch_pools_by_coins(&self, coin_a: &str, coin_b: &str, provider: Arc<dyn AlienProvider>) -> Result<Vec<CetusPool>, SwapperError> {
        let client = CetusClient::new(provider.clone());
        let pools = client
            .get_pool_by_token(coin_a, coin_b)
            .await?
            .iter()
            .filter_map(|x| if x.object.is_pause { None } else { Some(CetusPool::from(x.clone())) })
            .collect::<Vec<CetusPool>>();

        Ok(pools)
    }

    async fn fetch_pool_by_id(&self, pool_id: &str, provider: Arc<dyn AlienProvider>) -> Result<CetusPoolObject, SwapperError> {
        let rpc = SuiRpc::GetObject(pool_id.into(), Some(ObjectDataOptions::default()));
        let response: JsonRpcResult<CetusPoolType> = jsonrpc_call(&rpc, provider, &Chain::Sui).await?;
        let object = response.take()?.data;
        Ok(object.content.unwrap().fields)
    }
}

#[async_trait]
impl GemSwapProvider for Cetus {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![SwapChainAsset::All(Chain::Sui)]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let from_coin = Cetus::get_coin_address(&request.from_asset);
        let to_coin = Cetus::get_coin_address(&request.to_asset);
        let pools = self.fetch_pools_by_coins(&from_coin, &to_coin, provider.clone()).await?;

        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let pool_id = &pools[0].pool_address;
        let pool_object = self.fetch_pool_by_id(pool_id, provider).await?;
        debug_println!("{:?}", pool_object);
        todo!()
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        Ok(true)
    }
}
