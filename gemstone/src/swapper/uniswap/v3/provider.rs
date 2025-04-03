use crate::{
    network::{jsonrpc::batch_jsonrpc_call, AlienProvider, JsonRpcError},
    swapper::{
        approval::{check_approval_erc20, check_approval_permit2},
        models::*,
        slippage::apply_slippage_in_bp,
        uniswap::{deadline::get_sig_deadline, fee_token::get_fee_token, quote_result::get_best_quote, swap_route::build_swap_route},
        weth_address, GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::EthereumRpc,
    uniswap::{command::encode_commands, path::get_base_pair},
};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::primitives::{hex::encode_prefixed as HexEncode, Address, Bytes, U256};
use async_trait::async_trait;
use std::{fmt::Debug, str::FromStr, sync::Arc};

use super::{commands::build_commands, path::build_paths_with_routes, UniversalRouterProvider, DEFAULT_SWAP_GAS_LIMIT};

#[derive(Debug)]
pub struct UniswapV3 {
    provider: Box<dyn UniversalRouterProvider>,
}

impl UniswapV3 {
    pub fn new(provider: Box<dyn UniversalRouterProvider>) -> Self {
        Self { provider }
    }

    pub fn support_chain(&self, chain: &Chain) -> bool {
        self.provider.get_deployment_by_chain(chain).is_some()
    }

    fn get_asset_address(asset: &AssetId, evm_chain: EVMChain) -> Result<EthereumAddress, SwapperError> {
        weth_address::parse_into_address(asset, evm_chain)
    }

    fn parse_request(request: &SwapQuoteRequest) -> Result<(EVMChain, EthereumAddress, EthereumAddress, U256), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::get_asset_address(&request.from_asset, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset, evm_chain)?;
        let amount_in = U256::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }

    async fn check_erc20_approval(
        &self,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<ApprovalType, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        // Check token allowance, spender is permit2
        check_approval_erc20(
            wallet_address.to_string(),
            token.to_string(),
            deployment.permit2.to_string(),
            amount,
            provider,
            chain,
        )
        .await
    }

    async fn check_permit2_approval(
        &self,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;

        Ok(check_approval_permit2(
            deployment.permit2,
            wallet_address.to_string(),
            token.to_string(),
            deployment.universal_router.to_string(),
            amount,
            provider.clone(),
            chain,
        )
        .await?
        .permit2_data())
    }
}

#[async_trait]
impl GemSwapProvider for UniswapV3 {
    fn provider(&self) -> &SwapProviderType {
        self.provider.provider()
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapChainAsset::All(*x)).collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Prevent swaps on unsupported chains
        if !self.support_chain(&request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }

        // Check deployment and weth contract
        let deployment = self
            .provider
            .get_deployment_by_chain(&request.from_asset.chain)
            .ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, amount_in) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let fee_tiers = self.provider.get_tiers();
        let base_pair = get_base_pair(&evm_chain, true).ok_or(SwapperError::ComputeQuoteError {
            msg: "base pair not found".into(),
        })?;

        let fee_preference = get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
        let fee_bps = request.options.clone().fee.unwrap_or_default().evm.bps;
        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            amount_in - amount_in * U256::from(fee_bps) / U256::from(10000)
        } else {
            amount_in
        };

        // Build paths for QuoterV2
        // [
        //     [direct_fee_tier1, ..., ..., ... ],
        //     [weth_hop_fee_tier1, ..., ..., ... ],
        //     [usdc_hop_fee_tier1, ..., ..., ... ],
        //     [...],
        // ]
        let paths_array = super::path::build_paths(&token_in, &token_out, &fee_tiers, &base_pair);
        let requests: Vec<_> = paths_array
            .iter()
            .map(|paths| {
                let calls: Vec<EthereumRpc> = paths
                    .iter()
                    .map(|path| super::quoter_v2::build_quoter_request(&request.mode, &request.wallet_address, deployment.quoter_v2, quote_amount_in, &path.1))
                    .collect();

                // batch fee_tiers.len() requests into one jsonrpc call
                batch_jsonrpc_call(calls, provider.clone(), &request.from_asset.chain)
            })
            .collect();

        // fire batch requests in parallel
        let batch_results: Vec<_> = futures::future::join_all(requests)
            .await
            .into_iter()
            .map(|r| r.map_err(JsonRpcError::from))
            .collect();

        let quote_result = get_best_quote(&batch_results, super::quoter_v2::decode_quoter_response)?;

        let to_value = if fee_preference.is_input_token {
            quote_result.amount_out
        } else {
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let fee_tier_idx = quote_result.fee_tier_idx;
        let batch_idx = quote_result.batch_idx;
        let gas_estimate = quote_result.gas_estimate;

        // construct routes
        let fee_tier: u32 = fee_tiers[fee_tier_idx % fee_tiers.len()].clone() as u32;
        let asset_id_in = AssetId::from(request.from_asset.chain, Some(token_in.to_checksum()));
        let asset_id_out = AssetId::from(request.to_asset.chain, Some(token_out.to_checksum()));
        let asset_id_intermediary: Option<AssetId> = match batch_idx {
            // direct route
            0 => None,
            // 2 hop route with intermediary token
            _ => {
                let first_token_out = &paths_array[batch_idx][0].0[0].token_out;
                Some(AssetId::from(request.to_asset.chain, Some(first_token_out.to_checksum())))
            }
        };
        let routes = build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &fee_tier.to_string(), gas_estimate);

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: routes.clone(),
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
        })
    }

    async fn fetch_permit2_for_quote(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        if quote.request.from_asset.is_native() {
            return Ok(None);
        }
        let wallet_address: Address = quote.request.wallet_address.as_str().parse().map_err(SwapperError::from)?;
        let (_, token_in, _, amount_in) = Self::parse_request(&quote.request)?;
        self.check_permit2_approval(wallet_address, &token_in.to_checksum(), amount_in, &quote.request.from_asset.chain, provider)
            .await
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = self
            .provider
            .get_deployment_by_chain(&request.from_asset.chain)
            .ok_or(SwapperError::NotSupportedChain)?;
        let to_amount = U256::from_str(&quote.to_value).map_err(|_| SwapperError::InvalidAmount)?;
        let wallet_address: Address = request.wallet_address.as_str().parse().map_err(SwapperError::from)?;
        let permit = data.permit2_data().map(|data| data.into());

        let mut gas_limit: Option<String> = None;
        let approval: Option<ApprovalData> = if quote.request.from_asset.is_native() {
            None
        } else {
            // Check if need to approve permit2 contract
            self.check_erc20_approval(wallet_address, &token_in.to_checksum(), amount_in, &request.from_asset.chain, provider)
                .await?
                .approval_data()
        };
        if approval.is_some() {
            gas_limit = Some(DEFAULT_SWAP_GAS_LIMIT.to_string());
        }

        let sig_deadline = get_sig_deadline();

        let evm_chain = EVMChain::from_chain(quote.request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = get_base_pair(&evm_chain, true);
        let fee_preference = get_fee_token(&request.mode, base_pair.as_ref(), &token_in, &token_out);

        let path: Bytes = build_paths_with_routes(&quote.data.routes)?;
        let commands = build_commands(
            request,
            &token_in,
            &token_out,
            amount_in,
            to_amount,
            &path,
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
