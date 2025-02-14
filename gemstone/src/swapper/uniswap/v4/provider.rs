use crate::{
    network::{batch_jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{
        uniswap::{fee_token::get_fee_token, swap_route::build_swap_route},
        weth_address, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest,
        SwapperError,
    },
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::EthereumRpc,
    uniswap::{deployment, path::get_base_pair, FeeTier},
};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::primitives::U256;
use async_trait::async_trait;
use std::{str::FromStr, sync::Arc, vec};

use super::path::build_pool_keys;

#[derive(Debug, Default)]
pub struct UniswapV4 {}

impl UniswapV4 {
    pub fn boxed() -> Box<dyn GemSwapProvider> {
        Box::new(Self::default())
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        deployment::v4::get_uniswap_router_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn parse_asset_address(asset: &AssetId, evm_chain: EVMChain) -> Result<EthereumAddress, SwapperError> {
        if asset.is_native() {
            Ok(EthereumAddress { bytes: vec![0u8; 20] })
        } else {
            weth_address::parse_into_address(asset, evm_chain)
        }
    }

    fn parse_request(request: &SwapQuoteRequest) -> Result<(EVMChain, EthereumAddress, EthereumAddress, u128), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::parse_asset_address(&request.from_asset, evm_chain)?;
        let token_out = Self::parse_asset_address(&request.to_asset, evm_chain)?;
        let amount_in = u128::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        Ok((evm_chain, token_in, token_out, amount_in))
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
    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Check deployment and weth contract
        let deployment = deployment::v4::get_uniswap_router_deployment_by_chain(&request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, amount_in) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let fee_tiers = self.get_tiers();
        let base_pair = get_base_pair(&evm_chain).ok_or(SwapperError::ComputeQuoteError {
            msg: "base pair not found".into(),
        })?;

        let fee_preference = get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
        let fee_bps = request.options.clone().fee.unwrap_or_default().evm.bps;
        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            amount_in - amount_in * u128::from(fee_bps) / 10000_u128
        } else {
            amount_in
        };

        // Build PoolKeys for Quoter
        // [
        //     [direct_fee_tier1, ..., ..., ... ],
        //     [weth_hop_fee_tier1, ..., ..., ... ],
        //     [usdc_hop_fee_tier1, ..., ..., ... ],
        // ]
        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);
        let calls: Vec<EthereumRpc> = pool_keys
            .iter()
            .map(|pool_key| super::quoter::build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        let batch_call = batch_jsonrpc_call(calls, provider.clone(), &request.from_asset.chain);
        let requests = vec![batch_call];

        // fire batch requests in parallel
        let batch_results: Vec<_> = futures::future::join_all(requests).await.into_iter().collect();

        let mut max_amount_out: Option<U256> = None;
        let mut batch_idx = 0;
        let mut fee_tier_idx = 0;
        let mut gas_estimate: Option<String> = None;

        for (batch, batch_result) in batch_results.iter().enumerate() {
            match batch_result {
                Ok(results) => {
                    for (index, result) in results.iter().enumerate() {
                        match result {
                            JsonRpcResult::Value(value) => {
                                let quoter_tuple = super::quoter::decode_quoter_response(&value)?;
                                if quoter_tuple.0 > max_amount_out.unwrap_or_default() {
                                    max_amount_out = Some(quoter_tuple.0);
                                    fee_tier_idx = index;
                                    batch_idx = batch;
                                    gas_estimate = Some(quoter_tuple.1.to_string());
                                }
                            }
                            _ => continue, // skip no pool error etc.
                        }
                    }
                }
                _ => continue, // skip jsonrpc call error
            }
        }

        if max_amount_out.is_none() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        // construct routes
        let fee_tier: u32 = fee_tiers[fee_tier_idx % fee_tiers.len()].clone() as u32;
        let asset_id_in = AssetId::from(request.from_asset.chain, Some(token_in.to_checksum()));
        let asset_id_out = AssetId::from(request.to_asset.chain, Some(token_out.to_checksum()));
        let asset_id_intermediary: Option<AssetId> = match batch_idx {
            // direct route
            0 => None,
            // 2 hop route with intermediary token
            _ => {
                let first_token_out = &pool_keys[batch_idx].0[0].token_out;
                Some(AssetId::from(request.to_asset.chain, Some(first_token_out.to_checksum())))
            }
        };
        let routes = build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &fee_tier.to_string(), gas_estimate);

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: max_amount_out.unwrap().to_string(), // safe to unwrap here because we will early return if no quote is available
            data: SwapProviderData {
                provider: self.provider(),
                routes: routes.clone(),
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
        })
    }
    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}
