use crate::{
    FetchQuoteData, Permit2ApprovalData, Swapper, SwapperError, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData, SwapperQuoteRequest,
    alien::factory::JsonRpcClientFactory,
    approval::{check_approval_erc20_with_client, check_approval_permit2_with_client},
    eth_address,
    models::*,
    slippage::apply_slippage_in_bp,
    uniswap::{
        deadline::get_sig_deadline,
        fee_token::get_fee_token,
        quote_result::get_best_quote,
        swap_route::{RouteData, build_swap_route},
    },
};
use alloy_primitives::{Address, Bytes, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{command::encode_commands, path::get_base_pair},
};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};
use std::{fmt, fmt::Debug, marker::PhantomData, str::FromStr, sync::Arc};

use super::{DEFAULT_SWAP_GAS_LIMIT, UniversalRouterProvider, commands::build_commands, path::build_paths_with_routes};

pub struct UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    provider: Box<dyn UniversalRouterProvider>,
    rpc_factory: Arc<F>,
    phantom: PhantomData<C>,
}

impl<C, F> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    pub fn new(provider: Box<dyn UniversalRouterProvider>, rpc_factory: Arc<F>) -> Self {
        Self {
            provider,
            rpc_factory,
            phantom: PhantomData,
        }
    }

    pub fn support_chain(&self, chain: &Chain) -> bool {
        self.provider.get_deployment_by_chain(chain).is_some()
    }

    fn get_asset_address(asset_id: &str, evm_chain: EVMChain) -> Result<Address, SwapperError> {
        let asset_id = AssetId::new(asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        eth_address::normalize_weth_address(&asset_id, evm_chain)
    }

    fn parse_request(request: &SwapperQuoteRequest) -> Result<(EVMChain, Address, Address, U256), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::get_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = U256::from_str(&request.value).map_err(SwapperError::from)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }

    async fn check_erc20_approval(
        &self,
        client: &JsonRpcClient<C>,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
    ) -> Result<ApprovalType, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let spender = deployment.permit2.to_string();
        check_approval_erc20_with_client(wallet_address.to_string(), token.to_string(), spender, amount, client).await
    }

    async fn check_permit2_approval(
        &self,
        client: &JsonRpcClient<C>,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
    ) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;

        Ok(check_approval_permit2_with_client(
            deployment.permit2,
            wallet_address.to_string(),
            token.to_string(),
            deployment.universal_router.to_string(),
            amount,
            client,
        )
        .await?
        .permit2_data())
    }
}

impl<C, F> fmt::Debug for UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV3").finish()
    }
}

#[async_trait]
impl<C, F> Swapper for UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    fn provider(&self) -> &SwapperProviderType {
        self.provider.provider()
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all()
            .iter()
            .filter(|x| self.support_chain(x))
            .map(|x| SwapperChainAsset::All(*x))
            .collect()
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let client = self.rpc_factory.client_for(from_chain).map_err(SwapperError::from)?;

        let fee_tiers = self.provider.get_tiers();
        let base_pair = get_base_pair(&evm_chain, true).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;

        let fee_preference = get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
        let fee_bps = request.options.clone().fee.unwrap_or_default().evm.bps;

        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
        };

        let paths_array = super::path::build_paths(&token_in, &token_out, &fee_tiers, &base_pair);
        let requests: Vec<_> = paths_array
            .iter()
            .map(|paths| {
                let client = client.clone();
                let calls: Vec<EthereumRpc> = paths
                    .iter()
                    .map(|path| super::quoter_v2::build_quoter_request(&request.mode, &request.wallet_address, deployment.quoter_v2, quote_amount_in, &path.1))
                    .collect();
                async move { client.batch_call_requests(calls).await }
            })
            .collect();

        let batch_results = futures::future::join_all(requests).await;

        let quote_result = get_best_quote(&batch_results, super::quoter_v2::decode_quoter_response)?;

        let to_value = if fee_preference.is_input_token {
            quote_result.amount_out
        } else {
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let to_min_value = apply_slippage_in_bp(&to_value, request.options.slippage.bps);

        let fee_tier_idx = quote_result.fee_tier_idx;
        let batch_idx = quote_result.batch_idx;
        let gas_estimate = quote_result.gas_estimate;

        let fee_tier: u32 = fee_tiers[fee_tier_idx % fee_tiers.len()] as u32;
        let asset_id_in = AssetId::from(from_chain, Some(token_in.to_checksum(None)));
        let asset_id_out = AssetId::from(to_chain, Some(token_out.to_checksum(None)));
        let asset_id_intermediary: Option<AssetId> = match batch_idx {
            0 => None,
            _ => {
                let first_token_out = &paths_array[batch_idx][0].0[0].token_out;
                Some(AssetId::from(to_chain, Some(first_token_out.to_checksum(None))))
            }
        };
        let route_data = RouteData {
            fee_tier: fee_tier.to_string(),
            min_amount_out: to_min_value.to_string(),
        };
        let routes = build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &route_data, gas_estimate);

        Ok(SwapperQuote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: SwapperProviderData {
                provider: self.provider().clone(),
                routes: routes.clone(),
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn fetch_permit2_for_quote(&self, quote: &SwapperQuote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        if from_asset.is_native() {
            return Ok(None);
        }
        let client = self.rpc_factory.client_for(from_asset.chain).map_err(SwapperError::from)?;
        let wallet_address = eth_address::parse_str(&quote.request.wallet_address)?;
        let (_, token_in, _, amount_in) = Self::parse_request(&quote.request)?;
        self.check_permit2_approval(&client, wallet_address, &token_in.to_checksum(None), amount_in, &from_asset.chain)
            .await
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_chain = request.from_asset.chain();
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = self.rpc_factory.client_for(from_chain).map_err(SwapperError::from)?;

        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = U256::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let wallet_address = eth_address::parse_str(&request.wallet_address)?;
        let permit = data.permit2_data().map(|data| data.into());

        let mut gas_limit: Option<String> = None;
        let approval: Option<ApprovalData> = if quote.request.from_asset.is_native() {
            None
        } else {
            self.check_erc20_approval(&client, wallet_address, &token_in.to_checksum(None), amount_in, &from_chain)
                .await?
                .approval_data()
        };
        if approval.is_some() {
            gas_limit = Some(DEFAULT_SWAP_GAS_LIMIT.to_string());
        }

        let sig_deadline = get_sig_deadline();

        let evm_chain = EVMChain::from_chain(from_chain).ok_or(SwapperError::NotSupportedChain)?;
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

        Ok(SwapperQuoteData {
            to: deployment.universal_router.into(),
            value,
            data: HexEncode(encoded),
            approval,
            gas_limit,
        })
    }
}
