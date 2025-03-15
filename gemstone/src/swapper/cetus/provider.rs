use alloy_core::primitives::U256;
use async_trait::async_trait;
use bcs;
use futures::join;
use num_bigint::BigInt;
use num_traits::{ToBytes, ToPrimitive};
use std::{str::FromStr, sync::Arc};

use super::{
    client::{models::CetusPool, CetusClient},
    clmm::{
        tx_builder::{CetusConfig, SharedObject, SwapParams},
        TickData, TransactionBuilder,
    },
    models::{CalculatedSwapResult, CetusPoolObject, CetusPoolType, RoutePoolData},
    CETUS_CLMM_PACKAGE_ID, CETUS_GLOBAL_CONFIG_ID, CETUS_GLOBAL_CONFIG_SHARED_VERSION, CETUS_ROUTER_PACKAGE_ID,
};
use crate::{
    network::AlienProvider,
    sui::rpc::{models::InspectEvent, SuiClient},
    swapper::{
        slippage::apply_slippage_in_bp, FetchQuoteData, GemSwapMode, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType,
        SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};
use gem_sui::{
    jsonrpc::{ObjectDataOptions, SuiData, SuiRpc},
    model::TxOutput,
    EMPTY_ADDRESS, SUI_COIN_TYPE_FULL,
};
use primitives::{AssetId, Chain};
use sui_types::{
    base_types::ObjectID,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Command, ObjectArg, TransactionData, TransactionKind},
    Identifier, TypeTag,
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

    pub fn get_clmm_config(&self) -> Result<CetusConfig, SwapperError> {
        Ok(CetusConfig {
            global_config: SharedObject {
                id: ObjectID::from_str(CETUS_GLOBAL_CONFIG_ID).unwrap(),
                shared_version: CETUS_GLOBAL_CONFIG_SHARED_VERSION,
            },
            clmm_pool: ObjectID::from_str(CETUS_CLMM_PACKAGE_ID).unwrap(),
            router: ObjectID::from_str(CETUS_ROUTER_PACKAGE_ID).unwrap(),
        })
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

    async fn pre_swap(
        &self,
        pool: &CetusPool,
        pool_obj: &SharedObject,
        a2b: bool,
        buy_amount_in: bool,
        amount: BigInt,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<CalculatedSwapResult, anyhow::Error> {
        let client = SuiClient::new(provider);
        let mut ptb = ProgrammableTransactionBuilder::new();
        let type_args = vec![TypeTag::from_str(&pool.coin_a_address)?, TypeTag::from_str(&pool.coin_b_address)?];
        let args = [
            ptb.obj(ObjectArg::SharedObject {
                id: pool_obj.id,
                initial_shared_version: pool_obj.initial_shared_version(),
                mutable: false,
            })?,
            ptb.pure(a2b)?,
            ptb.pure(buy_amount_in)?,
            ptb.pure(amount.to_u64().unwrap_or(0))?,
        ];
        let move_call = Command::move_call(
            ObjectID::from_str(CETUS_ROUTER_PACKAGE_ID)?,
            Identifier::from_str("fetcher_script")?,
            Identifier::from_str("calculate_swap_result")?,
            type_args,
            args.to_vec(),
        );
        ptb.command(move_call);
        let tx = ptb.finish();
        let tx_kind = TransactionKind::ProgrammableTransaction(tx);
        let tx_bytes = bcs::to_bytes(&tx_kind).unwrap();
        let result = client.inspect_tx_block(EMPTY_ADDRESS, &tx_bytes).await?;
        let event = result.events.as_array().map(|x| x.first().unwrap()).unwrap();
        let event_data: InspectEvent<SuiData<CalculatedSwapResult>> = serde_json::from_value(event.clone()).unwrap();
        Ok(event_data.parsed_json.data)
    }

    #[allow(unused)]
    async fn fetch_ticks_by_pool_id(
        &self,
        pool: &CetusPoolObject,
        pool_obj: &SharedObject,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<Vec<TickData>, anyhow::Error> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        let type_args = vec![TypeTag::from_str(&pool.coin_a)?, TypeTag::from_str(&pool.coin_b)?];
        let start = ptb.command(Command::make_move_vec(Some(TypeTag::U32), vec![]));
        let limit = ptb.pure(512)?;
        let args = [
            ptb.obj(ObjectArg::SharedObject {
                id: pool_obj.id,
                initial_shared_version: pool_obj.initial_shared_version(),
                mutable: false,
            })?,
            start,
            limit,
        ];
        let move_call = Command::move_call(
            ObjectID::from_str(CETUS_ROUTER_PACKAGE_ID)?,
            Identifier::from_str("fetcher_script")?,
            Identifier::from_str("get_ticks")?,
            type_args,
            args.to_vec(),
        );
        ptb.command(move_call);
        todo!()
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
        if request.from_asset.chain != Chain::Sui || request.to_asset.chain != Chain::Sui {
            return Err(SwapperError::NotSupportedChain);
        }

        let from_coin = Self::get_coin_address(&request.from_asset);
        let to_coin = Self::get_coin_address(&request.to_asset);
        let amount_in = BigInt::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        let pools = self.fetch_pools_by_coins(&from_coin, &to_coin, provider.clone()).await?;
        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let pool = &pools[0]; // FIXME pick multiple pools
        let sui_client = SuiClient::new(provider.clone());
        let pool_data: CetusPoolType = sui_client
            .rpc_call(SuiRpc::GetObject(pool.address.to_string(), Some(ObjectDataOptions::default())))
            .await?;
        let pool_shared = SharedObject {
            id: pool_data.data.object_id,
            shared_version: pool_data.data.initial_shared_version().expect("Initial shared version should be available"),
        };

        let a2b = pool.coin_a_address == from_coin;
        let buy_amount_in = request.mode == GemSwapMode::ExactIn;

        // Call router to pre-swap
        let swap_result = self
            .pre_swap(pool, &pool_shared, a2b, buy_amount_in, amount_in, provider.clone())
            .await
            .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        let quote_amount = U256::from_le_slice(swap_result.amount_out.to_le_bytes().as_slice());
        let slippage_bps = request.options.slippage.bps;
        let expect_min = apply_slippage_in_bp(&quote_amount, slippage_bps);

        // Prepare route data
        let route_data = RoutePoolData {
            object_id: pool_data.data.object_id,
            version: pool_data.data.version,
            digest: pool_data.data.digest,
            coin_a: pool.coin_a_address.clone(),
            coin_b: pool.coin_b_address.clone(),
            initial_shared_version: pool_shared.shared_version,
        };

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_amount.to_string(),
            to_min_value: expect_min.to_string(),
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![SwapRoute {
                    input: AssetId::from(Chain::Sui, Some(from_coin.clone())),
                    output: AssetId::from(Chain::Sui, Some(to_coin.clone())),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        // Validate quote data
        let route = &quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let sender_address = quote.request.wallet_address.parse().map_err(|_| SwapperError::InvalidAddress {
            address: quote.request.wallet_address.clone(),
        })?;
        let route_data: RoutePoolData = serde_json::from_str(&route.route_data).map_err(|e| SwapperError::TransactionError {
            msg: format!("Invalid route data: {}", e),
        })?;

        let from_asset = &route.input;
        let from_coin = Self::get_coin_address(from_asset);
        let sui_client = SuiClient::new(provider.clone());
        let cetus_config = self.get_clmm_config()?;

        // Execute gas_price and coin_assets fetching in parallel
        let (gas_price_result, all_coin_assets_result) = join!(sui_client.get_gas_price(), sui_client.get_coin_assets(sender_address));

        let gas_price = gas_price_result.map_err(SwapperError::from)?;
        let all_coin_assets = all_coin_assets_result.map_err(SwapperError::from)?;

        // Prepare swap params for tx building
        let a2b = from_coin == route_data.coin_a;
        let swap_params = SwapParams {
            pool_object_shared: SharedObject {
                id: route_data.object_id,
                shared_version: route_data.initial_shared_version,
            },
            a2b,
            by_amount_in: quote.request.mode == GemSwapMode::ExactIn,
            amount: BigInt::from_str(&quote.from_value)?,
            amount_limit: BigInt::from_str(&quote.to_min_value)?,
            coin_type_a: route_data.coin_a.clone(),
            coin_type_b: route_data.coin_b.clone(),
            swap_partner: None, // No swap partner for now
        };

        // Build tx
        let ptb = TransactionBuilder::build_swap_transaction(&cetus_config, &swap_params, &all_coin_assets).map_err(|e| SwapperError::TransactionError {
            msg: format!("Failed to build swap transaction: {}", e),
        })?;
        let tx = ptb.finish();

        // Estimate gas_budget
        let tx_kind = TransactionKind::ProgrammableTransaction(tx.clone());
        let tx_bytes = bcs::to_bytes(&tx_kind).map_err(|e| SwapperError::TransactionError { msg: e.to_string() })?;
        let inspect_result = sui_client.inspect_tx_block(EMPTY_ADDRESS, &tx_bytes).await?;
        let gas_budget = (inspect_result.effects.total_gas_cost() as f64 * 1.2) as u64;

        let coin_refs = all_coin_assets
            .iter()
            .filter(|x| x.coin_type == SUI_COIN_TYPE_FULL)
            .map(|x| x.to_ref())
            .collect::<Vec<_>>();

        let tx_data = TransactionData::new_programmable(sender_address, coin_refs, tx, gas_budget, gas_price);
        let tx_output = TxOutput::from_tx_data(&tx_data).unwrap();

        Ok(SwapQuoteData {
            to: "".to_string(),
            value: "".to_string(),
            data: tx_output.base64_encoded(),
            approval: None,
            gas_limit: None,
        })
    }
}
