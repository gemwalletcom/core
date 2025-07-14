use alloy_primitives::{hex::encode_prefixed as HexEncode, Address, U256};
use async_trait::async_trait;
use std::{str::FromStr, sync::Arc, vec};

use crate::{
    network::{AlienProvider, JsonRpcClient, JsonRpcError},
    swapper::{
        approval::{check_approval_erc20, check_approval_permit2},
        eth_address,
        slippage::apply_slippage_in_bp,
        uniswap::{
            deadline::get_sig_deadline,
            fee_token::get_fee_token,
            quote_result::get_best_quote,
            swap_route::{build_swap_route, get_intermediaries, RouteData},
        },
        FetchQuoteData, GemApprovalData, GemSwapProvider, Permit2ApprovalData, SwapChainAsset, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData,
        SwapQuoteRequest, Swapper, SwapperError,
    },
};
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{
        command::encode_commands,
        contracts::v4::IV4Quoter::QuoteExactParams,
        deployment::v4::get_uniswap_deployment_by_chain,
        path::{get_base_pair, TokenPair},
        FeeTier,
    },
};
use primitives::{AssetId, Chain, EVMChain};

use super::{
    commands::build_commands,
    path::{build_pool_keys, build_quote_exact_params},
    quoter::{build_quote_exact_requests, build_quote_exact_single_request},
    DEFAULT_SWAP_GAS_LIMIT,
};

#[derive(Debug)]
pub struct UniswapV4 {
    pub provider: SwapProviderType,
}

impl Default for UniswapV4 {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(GemSwapProvider::UniswapV4),
        }
    }
}

impl UniswapV4 {
    pub fn boxed() -> Box<dyn Swapper> {
        Box::new(Self::default())
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        get_uniswap_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn is_base_pair(token_in: &Address, token_out: &Address, evm_chain: &EVMChain) -> bool {
        let base_set = get_base_pair(evm_chain, false).unwrap().to_set();
        base_set.contains(token_in) || base_set.contains(token_out)
    }

    fn parse_asset_address(asset_id: &str, _evm_chain: EVMChain) -> Result<Address, SwapperError> {
        let asset_id = AssetId::new(asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        if asset_id.is_native() {
            Ok(Address::ZERO)
        } else {
            eth_address::parse_asset_id(&asset_id)
        }
    }

    fn parse_request(request: &SwapQuoteRequest) -> Result<(EVMChain, Address, Address, u128), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::parse_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::parse_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = u128::from_str(&request.value).map_err(SwapperError::from)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }
}

#[async_trait]
impl Swapper for UniswapV4 {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }
    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapChainAsset::All(*x)).collect()
    }
    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        // Check deployment and weth contract
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let fee_tiers = self.get_tiers();
        let base_pair = get_base_pair(&evm_chain, false).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let fee_preference = get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
        let fee_bps = request.options.clone().fee.unwrap_or_default().evm.bps;
        // If fees are taken from input token, we need to use remaining amount as quote amount
        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
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
            .map(|pool_key| build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        let client = JsonRpcClient::new_with_chain(provider.clone(), from_chain);
        let batch_call = client.batch_call(calls);
        let mut requests = vec![batch_call];

        let quote_exact_params: Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>>;
        if !Self::is_base_pair(&token_in, &token_out, &evm_chain) {
            let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
            quote_exact_params = build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries);
            build_quote_exact_requests(deployment.quoter, &quote_exact_params)
                .iter()
                .for_each(|call_array| {
                    let batch_call = client.batch_call(call_array.to_vec());
                    requests.push(batch_call);
                });
        } else {
            quote_exact_params = vec![];
        }

        // fire batch requests in parallel
        let batch_results: Vec<_> = futures::future::join_all(requests)
            .await
            .into_iter()
            .map(|r| r.map_err(JsonRpcError::from))
            .collect();

        let quote_result = get_best_quote(&batch_results, super::quoter::decode_quoter_response)?;

        let fee_tier_idx = quote_result.fee_tier_idx;
        let batch_idx = quote_result.batch_idx;
        let gas_estimate = quote_result.gas_estimate;

        let to_value = if fee_preference.is_input_token {
            // fees are taken from input token
            quote_result.amount_out
        } else {
            // fees are taken from output tokene
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let to_min_value = apply_slippage_in_bp(&to_value, request.options.slippage.bps);

        // construct routes
        let fee_tier: u32 = fee_tiers[fee_tier_idx % fee_tiers.len()] as u32;
        let asset_id_in = AssetId::from(from_chain, Some(token_in.to_checksum(None)));
        let asset_id_out = AssetId::from(to_chain, Some(token_out.to_checksum(None)));
        let asset_id_intermediary: Option<AssetId> = match batch_idx {
            // direct route
            0 => None,
            // 2 hop route with intermediary token
            _ => {
                let first_token_out = &quote_exact_params[batch_idx][0].0[0].token_out;
                Some(AssetId::from(to_chain, Some(first_token_out.to_checksum(None))))
            }
        };
        let route_data = RouteData {
            fee_tier: fee_tier.to_string(),
            min_amount_out: to_min_value.to_string(),
        };
        let routes = build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &route_data, gas_estimate);

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: routes.clone(),
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn fetch_permit2_for_quote(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        if from_asset.is_native() {
            return Ok(None);
        }
        let (_, token_in, _, amount_in) = Self::parse_request(&quote.request)?;
        let v4_deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;

        let permit2_data = check_approval_permit2(
            v4_deployment.permit2,
            quote.request.wallet_address.clone(),
            token_in.to_string(),
            v4_deployment.universal_router.to_string(),
            U256::from(amount_in),
            provider.clone(),
            &from_asset.chain,
        )
        .await?
        .permit2_data();

        Ok(permit2_data)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let permit = data.permit2_data().map(|data| data.into());

        let mut gas_limit: Option<String> = None;
        let approval: Option<GemApprovalData> = if quote.request.from_asset.is_native() {
            None
        } else {
            // Check if need to approve permit2 contract
            check_approval_erc20(
                request.wallet_address.clone(),
                token_in.to_string(),
                deployment.permit2.to_string(),
                U256::from(amount_in),
                provider,
                &from_asset.chain,
            )
            .await?
            .approval_data()
        };
        if approval.is_some() {
            gas_limit = Some(DEFAULT_SWAP_GAS_LIMIT.to_string());
        }

        let sig_deadline = get_sig_deadline();
        let evm_chain = EVMChain::from_chain(from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = get_base_pair(&evm_chain, false);
        let fee_preference = get_fee_token(&request.mode, base_pair.as_ref(), &token_in, &token_out);

        let commands = build_commands(
            request,
            &token_in,
            &token_out,
            amount_in,
            to_amount,
            &quote.data.routes,
            permit,
            fee_preference.is_input_token,
        )?;
        let encoded = encode_commands(&commands, U256::from(sig_deadline));

        let wrap_input_eth = request.from_asset.is_native();
        let value = if wrap_input_eth { request.value.clone() } else { String::from("0") };

        Ok(SwapQuoteData {
            to: deployment.universal_router.into(),
            value,
            data: HexEncode(encoded),
            approval,
            gas_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::swapper::{GemSwapMode, GemSwapOptions};

    use super::*;

    #[test]
    fn test_is_base_pair() {
        let request = SwapQuoteRequest {
            from_asset: AssetId::from(Chain::SmartChain, Some("0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string())).into(),
            to_asset: AssetId::from_chain(Chain::SmartChain).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "40000000000000000".into(), // 0.04 Cake
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions::default(),
        };

        let (evm_chain, token_in, token_out, _) = UniswapV4::parse_request(&request).unwrap();

        assert!(UniswapV4::is_base_pair(&token_in, &token_out, &evm_chain));
    }
}
