use alloy_core::primitives::U256;
use async_trait::async_trait;
use num_bigint::BigInt;
use num_traits::ToBytes;
use std::{str::FromStr, sync::Arc};

use super::{
    client::CetusClient,
    clmm::compute_swap,
    models::{CetusPool, CetusPoolObject, CetusPoolType},
};
use crate::{
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{
        slippage::apply_slippage_in_bp, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote,
        SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};
use gem_sui::{
    jsonrpc::{ObjectDataOptions, SuiRpc},
    SUI_COIN_TYPE_FULL,
};
use primitives::{AssetId, Chain};

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
        let from_chain = request.from_asset.chain;
        let from_coin = Cetus::get_coin_address(&request.from_asset);
        let to_coin = Cetus::get_coin_address(&request.to_asset);
        let pools = self.fetch_pools_by_coins(&from_coin, &to_coin, provider.clone()).await?;

        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let pool = &pools[0]; // FIXME pick multiple pools
        let pool_object = self.fetch_pool_by_id(&pool.pool_address, provider).await?;
        let amount_in = BigInt::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        // Convert ticks to TickData format
        let tick_datas = &pool_object.tick_manager.fields.to_ticks();
        let pool_data = pool_object.clone().try_into()?;

        let a_to_b = pool.coin_type_a == from_coin;

        let swap_result = compute_swap(a_to_b, true, &amount_in, &pool_data, tick_datas).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;
        let quote_amount = U256::from_le_slice(swap_result.amount_out.to_le_bytes().as_slice());
        let slippage_bps = request.options.slippage.bps;
        let expect_min = apply_slippage_in_bp(&quote_amount, slippage_bps);

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_amount.to_string(),
            to_min_value: expect_min.to_string(),
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![SwapRoute {
                    input: AssetId::from(from_chain, Some(from_coin.clone())),
                    output: AssetId::from(from_chain, Some(to_coin.clone())),
                    route_data: serde_json::to_string(&pool_object).unwrap(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}
