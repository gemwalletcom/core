use alloy_core::primitives::U256;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bcs;
use futures::join;
use num_bigint::BigInt;
use num_traits::{ToBytes, ToPrimitive};
use std::{str::FromStr, sync::Arc};
use sui_transaction_builder::{unresolved::Input, Function, Serialized, TransactionBuilder as ProgrammableTransactionBuilder};
use sui_types::{Address, Identifier, ObjectId, TypeTag};

use super::{
    api::{models::CetusPool, CetusClient},
    models::{CalculatedSwapResult, CetusConfig, CetusPoolType, RoutePoolData, SharedObject, SwapParams},
    tx_builder::TransactionBuilder,
    CETUS_CLMM_PACKAGE_ID, CETUS_GLOBAL_CONFIG_ID, CETUS_GLOBAL_CONFIG_SHARED_VERSION, CETUS_MAINNET_PARTNER_ID, CETUS_PARTNER_SHARED_VERSION,
    CETUS_ROUTER_PACKAGE_ID,
};
use crate::{
    debug_println,
    network::AlienProvider,
    sui::{
        gas_budget::GasBudgetCalculator,
        rpc::{
            models::{InspectEvent, InspectResult},
            SuiClient,
        },
    },
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
                id: ObjectId::from_str(CETUS_GLOBAL_CONFIG_ID).unwrap(),
                shared_version: CETUS_GLOBAL_CONFIG_SHARED_VERSION,
            },
            partner: Some(SharedObject {
                id: ObjectId::from_str(CETUS_MAINNET_PARTNER_ID).unwrap(),
                shared_version: CETUS_PARTNER_SHARED_VERSION,
            }),
            clmm_pool: ObjectId::from_str(CETUS_CLMM_PACKAGE_ID).unwrap(),
            router: Address::from_str(CETUS_ROUTER_PACKAGE_ID).unwrap(),
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
        client: Arc<SuiClient>,
    ) -> Result<CalculatedSwapResult, anyhow::Error> {
        let call = self.pre_swap_call(pool, pool_obj, a2b, buy_amount_in, amount)?;
        let result: InspectResult = client.rpc_call(call).await?;
        self.decode_swap_result(&result)
    }

    fn pre_swap_call(&self, pool: &CetusPool, pool_obj: &SharedObject, a2b: bool, buy_amount_in: bool, amount: BigInt) -> Result<SuiRpc, anyhow::Error> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        let type_args = vec![TypeTag::from_str(&pool.coin_a_address)?, TypeTag::from_str(&pool.coin_b_address)?];

        let arguments = vec![
            ptb.input(Input::shared(pool_obj.id, pool_obj.shared_version, false)),
            ptb.input(Serialized(&a2b)),
            ptb.input(Serialized(&buy_amount_in)),
            ptb.input(Serialized(&amount.to_u64().unwrap_or(0))),
        ];
        let function = Function::new(
            Address::from_str(CETUS_ROUTER_PACKAGE_ID)?,
            Identifier::from_str("fetcher_script")?,
            Identifier::from_str("calculate_swap_result")?,
            type_args,
        );
        ptb.move_call(function, arguments);

        let tx_kind = ptb.finish()?.kind;
        let tx_bytes = bcs::to_bytes(&tx_kind).unwrap();
        Ok(SuiRpc::InspectTransactionBlock(EMPTY_ADDRESS.to_string(), STANDARD.encode(tx_bytes)))
    }

    fn decode_swap_result(&self, result: &InspectResult) -> Result<CalculatedSwapResult, anyhow::Error> {
        let event = result.events.as_array().map(|x| x.first().unwrap()).ok_or(SwapperError::ComputeQuoteError {
            msg: format!("Failed to get event from InspectResult: {:?}", result),
        })?;
        let event_data: InspectEvent<SuiData<CalculatedSwapResult>> = serde_json::from_value(event.clone())?;
        Ok(event_data.parsed_json.data)
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
        let from_coin = Self::get_coin_address(&request.from_asset);
        let to_coin = Self::get_coin_address(&request.to_asset);
        let amount_in = BigInt::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        let pools = self.fetch_pools_by_coins(&from_coin, &to_coin, provider.clone()).await?;
        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let buy_amount_in = request.mode == GemSwapMode::ExactIn;
        // Sort pools by liquidity and take top 2
        let mut sorted_pools = pools;
        sorted_pools.sort_by(|a, b| b.object.liquidity.cmp(&a.object.liquidity));
        let top_pools = sorted_pools.iter().take(2).collect::<Vec<_>>();

        // Create a single SuiClient that can be reused
        let sui_client = Arc::new(SuiClient::new(provider.clone()));

        let rpc_call = SuiRpc::GetMultipleObjects(
            top_pools.iter().map(|pool| pool.address.to_string()).collect(),
            Some(ObjectDataOptions::default()),
        );

        let pool_datas: Vec<CetusPoolType> = sui_client.rpc_call(rpc_call).await?;

        let pool_quotes = top_pools
            .into_iter()
            .zip(pool_datas.into_iter())
            .map(|(pool, pool_data)| {
                let shared_object = SharedObject {
                    id: pool_data.data.object_id,
                    shared_version: pool_data.data.initial_shared_version().expect("Initial shared version should be available"),
                };
                (pool, pool_data, shared_object, pool.coin_a_address == from_coin)
            })
            .collect::<Vec<_>>();

        if pool_quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        // Run pre-swap calculations in parallel using the same SuiClient instance
        let swap_futures = pool_quotes
            .iter()
            .map(|(pool, _, pool_shared, a2b)| self.pre_swap(pool, pool_shared, *a2b, buy_amount_in, amount_in.clone(), sui_client.clone()));
        let swap_results = futures::future::join_all(swap_futures).await;

        // Find the best quote
        let mut best_result: Option<CalculatedSwapResult> = None;
        let mut best_pool_data = None;
        let mut best_pool = None;

        for (result, (pool, pool_data, _, _)) in swap_results.into_iter().zip(pool_quotes.iter()) {
            if let Ok(swap_result) = result {
                let is_better = match &best_result {
                    None => true,
                    Some(best) => swap_result.amount_out > best.amount_out,
                };
                if is_better {
                    best_result = Some(swap_result);
                    best_pool_data = Some(pool_data.clone());
                    best_pool = Some(pool);
                }
            }
        }

        let (swap_result, pool_data, pool) = match (best_result, best_pool_data, best_pool) {
            (Some(r), Some(pd), Some(p)) => (r, pd, p),
            _ => return Err(SwapperError::NoQuoteAvailable),
        };

        let quote_amount = U256::from_le_slice(swap_result.amount_out.to_le_bytes().as_slice());
        let slippage_bps = request.options.slippage.bps;
        let fee_bps = 0; // request.options.fee.as_ref().map(|fee| fee.sui_cetus.bps).unwrap_or(0);
        let expect_min = apply_slippage_in_bp(&quote_amount, slippage_bps + fee_bps);

        // Prepare route data
        let route_data = RoutePoolData {
            object_id: pool_data.data.object_id,
            version: pool_data.data.version,
            digest: pool_data.data.digest,
            coin_a: pool.coin_a_address.clone(),
            coin_b: pool.coin_b_address.clone(),
            initial_shared_version: pool_data.data.initial_shared_version().expect("Initial shared version should be available"),
            fee_rate: pool.fee.to_string(),
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
            swap_partner: cetus_config.partner.clone(),
        };

        // Build tx
        let mut ptb =
            TransactionBuilder::build_swap_transaction(&cetus_config, &swap_params, &all_coin_assets).map_err(|e| SwapperError::TransactionError {
                msg: format!("Failed to build swap transaction: {}", e),
            })?;
        let tx = ptb.clone().finish().map_err(|e| SwapperError::TransactionError { msg: e.to_string() })?;

        // Estimate gas_budget
        let tx_kind = tx.kind.clone();
        let tx_bytes = bcs::to_bytes(&tx_kind).map_err(|e| SwapperError::TransactionError { msg: e.to_string() })?;
        let inspect_result = sui_client.inspect_tx_block(&quote.request.wallet_address, &tx_bytes).await?;
        let gas_budget = GasBudgetCalculator::gas_budget(&inspect_result.effects.gas_used);

        debug_println!("gas_budget: {:?}", gas_budget);

        let coin_refs = all_coin_assets
            .iter()
            .filter(|x| x.coin_type == SUI_COIN_TYPE_FULL)
            .map(|x| Input::immutable(x.coin_object_id, x.version, x.digest))
            .collect::<Vec<_>>();

        ptb.set_sender(sender_address);
        ptb.set_gas_budget(gas_budget);
        ptb.set_gas_price(gas_price);
        ptb.add_gas_objects(coin_refs);

        let tx_data = ptb.finish().map_err(|e| SwapperError::TransactionError { msg: e.to_string() })?;

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

#[cfg(test)]
mod tests {
    use sui_types::digests::ObjectDigest;

    use super::*;
    use crate::sui::{
        gas_budget,
        rpc::{models::InspectGasUsed, CoinAsset},
    };

    #[test]
    fn test_build_swap_transaction() {
        let provider = Cetus::default();
        let cetus_config = provider.get_clmm_config().unwrap();
        let sender_address = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".parse().unwrap();

        let route_data = include_str!("test/route_data.json");
        let route_data: RoutePoolData = serde_json::from_str(route_data).unwrap();
        let from_coin = SUI_COIN_TYPE_FULL;
        let a2b = from_coin == route_data.coin_a;

        let params = SwapParams {
            pool_object_shared: SharedObject {
                id: route_data.object_id,
                shared_version: route_data.initial_shared_version,
            },
            coin_type_a: route_data.coin_a,
            coin_type_b: route_data.coin_b,
            a2b,
            by_amount_in: true,
            amount: BigInt::from(1500000000),
            amount_limit: BigInt::from(3366366),
            swap_partner: cetus_config.partner.clone(),
        };

        let all_coins = vec![
            CoinAsset {
                coin_object_id: ObjectID::from_hex_literal("0xf16c8050267480b521889587515e40d10db27bf526b516b8c38421e5fa2c43e2").unwrap(),
                coin_type: SUI_COIN_TYPE_FULL.into(),
                digest: ObjectDigest::from_str("4Wr3NarWTJb2jpgtyE1siYxsB4EPtWuzCMCoYBQTXpkZ").unwrap(),
                balance: BigInt::from(1340089353u64),
                version: 508024613,
            },
            CoinAsset {
                coin_object_id: ObjectID::from_hex_literal("0xd8fd7990d0e74997ec0956be16336b9451cc29586ef224548d45e833ac926873").unwrap(),
                coin_type: SUI_COIN_TYPE_FULL.into(),
                digest: ObjectDigest::from_str("4yuV6Hjfe1cHnNhiB7MTkhGHtLxSVhmJ7pUuFQbzPpp").unwrap(),
                balance: BigInt::from(1011267243u64),
                version: 508024613,
            },
        ];
        let coin_refs = all_coins.iter().map(|x| x.to_ref()).collect::<Vec<_>>();

        let ptb = TransactionBuilder::build_swap_transaction(&cetus_config, &params, &all_coins).unwrap();
        let tx = ptb.finish();

        let tx_kind = TransactionKind::ProgrammableTransaction(tx.clone());
        let tx_bytes = bcs::to_bytes(&tx_kind).unwrap();
        let gas_used = InspectGasUsed {
            computation_cost: 750000,
            storage_cost: 16818800,
            storage_rebate: 14363316,
        };
        let gas_budget = gas_budget::GasBudgetCalculator::gas_budget(&gas_used);

        assert_eq!(STANDARD.encode(tx_bytes), "AAgACAAvaFkAAAAAAQHapGKSYyw8TY8x8j6g+bNqKP82d+loSYDkQ4QDpno9jy4FGAAAAAAAAQEBUeiDunwLVmomy8ipTNM+sKvUGKd8weYK0i/ZsfKc0qv7mnEWAAAAAAEBAQixh1tlQchH8F7XHQTLz6ZuToYZvzuJI7B8W1QJQzNmHn5DHgAAAAABAAEBAAjeXTMAAAAAAAAQrzMbqDJ/uzWxxP7/AAAAAAEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYBAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACBGNvaW4EemVybwEH26NGcuMMsGWx+T46tVMYdo/W/vZsFZQsn3y4RuL5AOcEdXNkYwRVU0RDAAACAAEBAAAAOlqpD/oz0JEA17aUHqHA/+arZudwYt3SYyDBsHOquxAOcG9vbF9zY3JpcHRfdjIVc3dhcF9iMmFfd2l0aF9wYXJ0bmVyAgfbo0Zy4wywZbH5Pjq1Uxh2j9b+9mwVlCyffLhG4vkA5wR1c2RjBFVTREMABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkACgEBAAECAAEDAAIAAAIBAAEEAAEAAAEFAAEGAAEHAA==");

        let tx_data = TransactionData::new_programmable(sender_address, coin_refs, tx, gas_budget, 750);
        let tx_output = TxOutput::from_tx_data(&tx_data).unwrap();

        assert_eq!(STANDARD.encode(tx_output.tx_data), "AAAIAAgAL2hZAAAAAAEB2qRikmMsPE2PMfI+oPmzaij/NnfpaEmA5EOEA6Z6PY8uBRgAAAAAAAEBAVHog7p8C1ZqJsvIqUzTPrCr1BinfMHmCtIv2bHynNKr+5pxFgAAAAABAQEIsYdbZUHIR/Be1x0Ey8+mbk6GGb87iSOwfFtUCUMzZh5+Qx4AAAAAAQABAQAI3l0zAAAAAAAAEK8zG6gyf7s1scT+/wAAAAABAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAQAAAAAAAAAAAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgRjb2luBHplcm8BB9ujRnLjDLBlsfk+OrVTGHaP1v72bBWULJ98uEbi+QDnBHVzZGMEVVNEQwAAAgABAQAAADpaqQ/6M9CRANe2lB6hwP/mq2bncGLd0mMgwbBzqrsQDnBvb2xfc2NyaXB0X3YyFXN3YXBfYjJhX3dpdGhfcGFydG5lcgIH26NGcuMMsGWx+T46tVMYdo/W/vZsFZQsn3y4RuL5AOcEdXNkYwRVU0RDAAcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgNzdWkDU1VJAAoBAQABAgABAwACAAACAQABBAABAAABBQABBgABBwCpvQST+b0feSpK7cH5nVRTWnWkbDj9VqjyxrfI11gXoQLxbIBQJnSAtSGIlYdRXkDRDbJ79Sa1FrjDhCHl+ixD4iXXRx4AAAAAIDQ4Wt7gZfnl48eOWKTFvCALpqZWU+Oid5hl4qpJAtEq2P15kNDnSZfsCVa+FjNrlFHMKVhu8iRUjUXoM6ySaHMl10ceAAAAACABBRXC6MyioZZDUf5P+MUg5XkPNgvIDl64t2PrOfOG1am9BJP5vR95KkrtwfmdVFNadaRsOP1WqPLGt8jXWBeh7gIAAAAAAAC0sToAAAAAAAA=");
        assert_eq!(hex::encode(tx_output.hash), "a3c4993de414195036774b8309d8af056180b2fc44d832e63b5ee8d26abf3701");
    }
}
