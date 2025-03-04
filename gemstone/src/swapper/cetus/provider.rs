use async_trait::async_trait;
use num_bigint::BigInt;
use std::{str::FromStr, sync::Arc};

use super::{
    client::CetusClient,
    math::{compute_swap, PoolData, TickData},
    models::{CetusPool, CetusPoolObject, CetusPoolType, TickManager},
};
use crate::{
    debug_println,
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{
        FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest,
        SwapperError,
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

    pub fn convert_to_tick_data(ticks: &TickManager) -> Vec<TickData> {
        let mut tick_data = Vec::new();
        let head_ticks = &ticks.ticks.fields.head;

        // Process each tick in the head vector
        for tick_obj in head_ticks {
            // Skip if the tick is None (OptionU64 represents None as bits = 0)
            if tick_obj.fields.is_none {
                continue;
            }

            // Convert tick index to i32
            let index: i32 = tick_obj.fields.v.parse().unwrap();

            // Calculate sqrt_price for this tick index
            // Using the formula: 1.0001^(tick/2) * 2^96
            // In fixed point notation: sqrt_price = 2^96 * (1.0001)^(tick/2)
            let sqrt_price = BigInt::from(2).pow(96) * BigInt::from(10001).pow((index / 2) as u32) / BigInt::from(10000).pow((index / 2) as u32);

            // Default liquidity_net to 1 since we don't have actual liquidity data
            // This is a simplification - in a real implementation you'd want to
            // calculate the actual liquidity net value
            let liquidity_net = BigInt::from(1);

            tick_data.push(TickData {
                index,
                sqrt_price,
                liquidity_net,
            });
        }
        tick_data
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

        let pool = &pools[0];
        let pool_object = self.fetch_pool_by_id(&pool.pool_address, provider).await?;
        let amount_in = BigInt::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        debug_println!("{:?}", pool_object);

        let slippage_bps = request.options.slippage.bps;
        let pool_data = PoolData {
            current_tick_index: pool_object.current_tick_index(),
            fee_rate: pool_object.fee_rate()?,
        };

        // Convert ticks to TickData format
        let swap_ticks = Self::convert_to_tick_data(&pool_object.tick_manager.fields);

        let a_to_b = pool.coin_type_a == from_coin;

        let swap_result = compute_swap(
            &pool_data,
            swap_ticks,
            pool_object.current_sqrt_price()?,
            pool_object.liquidity()?,
            amount_in,
            a_to_b,
            true, // by_amount_in
        );

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: swap_result.amount_out.to_string(),
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}
