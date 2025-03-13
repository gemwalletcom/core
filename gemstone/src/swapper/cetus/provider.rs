use alloy_core::primitives::U256;
use async_trait::async_trait;
use bcs;
use futures::join;
use num_bigint::BigInt;
use num_traits::ToBytes;
use std::{str::FromStr, sync::Arc};

use super::{
    client::{models::CetusPool, CetusClient},
    clmm::{
        compute_swap,
        tx_builder::{ClmmPoolConfig, IntegrateConfig, SwapParams},
        TransactionBuilder,
    },
    models::{get_pool_object, CetusPoolType, RoutePoolData},
    CETUS_CLMM_PACKAGE_ID, CETUS_CLMM_PUBLISHED_AT, CETUS_GLOBAL_CONFIG_ID, CETUS_GLOBAL_CONFIG_SHARED_VERSION, CETUS_INTEGRATE_PACKAGE_ID,
    CETUS_INTEGRATE_PUBLISHED_AT,
};
use crate::{
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    sui::rpc::SuiClient,
    swapper::{
        slippage::apply_slippage_in_bp, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote,
        SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};
use gem_sui::{
    jsonrpc::{ObjectDataOptions, SuiRpc},
    model::TxOutput,
    EMPTY_ADDRESS, SUI_COIN_TYPE_FULL,
};
use primitives::{AssetId, Chain};
use sui_types::{
    base_types::{ObjectID, SequenceNumber},
    transaction::TransactionData,
};

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

    pub fn get_clmm_config(&self) -> Result<(ClmmPoolConfig, IntegrateConfig), SwapperError> {
        Ok((
            ClmmPoolConfig {
                package_id: ObjectID::from_str(CETUS_CLMM_PACKAGE_ID).unwrap(),
                published_at: CETUS_CLMM_PUBLISHED_AT.to_string(),
                global_config_id: ObjectID::from_str(CETUS_GLOBAL_CONFIG_ID).unwrap(),
                global_config_shared_version: SequenceNumber::from_u64(CETUS_GLOBAL_CONFIG_SHARED_VERSION),
            },
            IntegrateConfig {
                package_id: ObjectID::from_str(CETUS_INTEGRATE_PACKAGE_ID).map_err(|_| SwapperError::TransactionError {
                    msg: "Invalid integrate package ID".to_string(),
                })?,
                published_at: CETUS_INTEGRATE_PUBLISHED_AT.to_string(),
            },
        ))
    }

    async fn fetch_pools_by_coins(&self, coin_a: &str, coin_b: &str, provider: Arc<dyn AlienProvider>) -> Result<Vec<CetusPool>, SwapperError> {
        let client = CetusClient::new(provider.clone());
        let pools = client
            .get_pool_by_token(coin_a, coin_b)
            .await?
            .iter()
            .filter_map(|x| if x.object.is_pause { None } else { Some(x.clone()) })
            .collect::<Vec<CetusPool>>();

        Ok(pools)
    }

    async fn fetch_pool_by_id(&self, pool_id: &str, provider: Arc<dyn AlienProvider>) -> Result<CetusPoolType, SwapperError> {
        let rpc = SuiRpc::GetObject(pool_id.into(), Some(ObjectDataOptions::default()));
        let response: JsonRpcResult<CetusPoolType> = jsonrpc_call(&rpc, provider, &Chain::Sui).await?;
        let object = response.take()?;
        Ok(object)
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
        let from_asset = &request.from_asset;
        let to_asset = &request.to_asset;
        let from_chain = from_asset.chain;
        let to_chain = to_asset.chain;
        if from_chain != Chain::Sui || to_chain != Chain::Sui {
            return Err(SwapperError::NotSupportedChain);
        }

        let from_coin = Self::get_coin_address(from_asset);
        let to_coin = Self::get_coin_address(to_asset);

        let pools = self.fetch_pools_by_coins(&from_coin, &to_coin, provider.clone()).await?;
        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let pool = &pools[0]; // FIXME pick multiple pools
        let pool_data = self.fetch_pool_by_id(&pool.address, provider).await?;
        let pool_object = get_pool_object(&pool_data).unwrap();
        let amount_in = BigInt::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        // Convert ticks to TickData format
        let tick_datas = &pool_object.tick_manager.fields.to_ticks();
        let clmm_data = pool_object.clone().try_into()?;

        let a2b = pool.coin_a_address == from_coin;

        let swap_result = compute_swap(a2b, true, &amount_in, &clmm_data, tick_datas).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;
        let quote_amount = U256::from_le_slice(swap_result.amount_out.to_le_bytes().as_slice());
        let slippage_bps = request.options.slippage.bps;
        let expect_min = apply_slippage_in_bp(&quote_amount, slippage_bps);
        let route_data = RoutePoolData {
            object_id: pool_data.data.object_id,
            version: pool_data.data.version,
            digest: pool_data.data.digest,
            coin_a: pool.coin_a_address.clone(),
            coin_b: pool.coin_b_address.clone(),
        };

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
                    route_data: serde_json::to_string(&route_data).unwrap(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        if quote.data.routes.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }

        let route = &quote.data.routes[0];
        let from_asset = &route.input;

        let sender_address = quote.request.wallet_address.parse().map_err(|_| SwapperError::InvalidAddress {
            address: quote.request.wallet_address.clone(),
        })?;
        let pool_data: RoutePoolData = serde_json::from_str(&route.route_data).map_err(|e| SwapperError::TransactionError {
            msg: format!("Invalid route data: {}", e),
        })?;
        let pool_ref = pool_data.obj_ref();
        let from_coin = Self::get_coin_address(from_asset);

        let sui_client = SuiClient::new(provider.clone());
        let (clmm_pool_config, integrate_config) = self.get_clmm_config()?;

        // Execute gas_price and coin_assets fetching in parallel
        let (gas_price_result, all_coin_assets_result) = join!(sui_client.get_gas_price(), sui_client.get_coin_assets(sender_address));

        let gas_price = gas_price_result.map_err(SwapperError::from)?;
        let all_coin_assets = all_coin_assets_result.map_err(SwapperError::from)?;

        let gas_coin = all_coin_assets
            .iter()
            .find(|x| x.coin_type == SUI_COIN_TYPE_FULL)
            .ok_or(SwapperError::TransactionError {
                msg: "Gas coin not found".to_string(),
            })?;

        let a2b = from_coin == pool_data.coin_a;
        let swap_params = SwapParams {
            pool_ref,
            a2b,
            by_amount_in: a2b,
            amount: BigInt::from_str(&quote.from_value)?,
            amount_limit: BigInt::from_str(&quote.to_min_value)?,
            coin_type_a: pool_data.coin_a.clone(),
            coin_type_b: pool_data.coin_b.clone(),
            swap_partner: None, // No swap partner for now
        };
        let ptb = TransactionBuilder::build_swap_transaction(&clmm_pool_config, &integrate_config, &swap_params, &all_coin_assets).map_err(|e| {
            SwapperError::TransactionError {
                msg: format!("Failed to build swap transaction: {}", e),
            }
        })?;
        let tx = ptb.finish();

        let dummy_tx_data = TransactionData::new_programmable(EMPTY_ADDRESS.parse().unwrap(), vec![gas_coin.to_ref()], tx.clone(), 50000000, gas_price);
        let tx_bytes = bcs::to_bytes(&dummy_tx_data).map_err(|e| SwapperError::TransactionError { msg: e.to_string() })?;
        let gas_budget = sui_client.estimate_gas_budget(EMPTY_ADDRESS, &tx_bytes).await?;

        let tx_data = TransactionData::new_programmable(sender_address, vec![gas_coin.to_ref()], tx, gas_budget, gas_price);
        let tx_output = TxOutput::from_tx_data(&tx_data).unwrap();

        Ok(SwapQuoteData {
            to: "".to_string(),
            value: "".to_string(),
            data: hex::encode(tx_output.tx_data),
            approval: None,
            gas_limit: None,
        })
    }
}
