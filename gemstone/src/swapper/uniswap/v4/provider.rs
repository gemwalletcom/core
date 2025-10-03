use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{
        FeeTier,
        command::encode_commands,
        contracts::v4::IV4Quoter::QuoteExactParams,
        deployment::v4::get_uniswap_deployment_by_chain,
        path::{TokenPair, get_base_pair},
    },
};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{AssetId, Chain, EVMChain};
use std::{collections::HashSet, fmt::{self, Debug}, marker::PhantomData, str::FromStr, sync::Arc, vec};

use crate::{
    models::GemApprovalData,
    network::{AlienClient, AlienEvmRpcFactory, AlienProvider, EvmRpcClientFactory},
    swapper::{
        FetchQuoteData, Permit2ApprovalData, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote,
        SwapperQuoteData, SwapperQuoteRequest,
        approval::{check_approval_erc20_with_client, check_approval_permit2_with_client},
        eth_address,
        models::ApprovalType,
        slippage::apply_slippage_in_bp,
        uniswap::{
            deadline::get_sig_deadline,
            fee_token::get_fee_token,
            quote_result::get_best_quote,
            swap_route::{RouteData, build_swap_route, get_intermediaries},
        },
    },
};

use super::{
    DEFAULT_SWAP_GAS_LIMIT,
    commands::build_commands,
    path::{build_pool_keys, build_quote_exact_params},
    quoter::{build_quote_exact_requests, build_quote_exact_single_request},
};

pub struct UniswapV4<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub provider: SwapperProviderType,
    rpc_factory: Arc<F>,
    _phantom: PhantomData<C>,
}

impl<C, F> fmt::Debug for UniswapV4<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV4")
            .field("provider", &self.provider)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swapper::{SwapperMode, SwapperOptions};
    use crate::network::{AlienClient, AlienEvmRpcFactory};

    #[test]
    fn test_is_base_pair() {
        let request = SwapperQuoteRequest {
            from_asset: AssetId::from(
                Chain::SmartChain,
                Some("0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string()),
            )
            .into(),
            to_asset: AssetId::from_chain(Chain::SmartChain).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "40000000000000000".into(),
            mode: SwapperMode::ExactIn,
            options: SwapperOptions::default(),
        };

        let (evm_chain, token_in, token_out, _) =
            UniswapV4::<AlienClient, AlienEvmRpcFactory>::parse_request(&request).unwrap();

        assert!(UniswapV4::<AlienClient, AlienEvmRpcFactory>::is_base_pair(
            &token_in,
            &token_out,
            &evm_chain,
        ));
    }
}

impl<C, F> UniswapV4<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    pub fn with_factory(factory: Arc<F>) -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::UniswapV4),
            rpc_factory: factory,
            _phantom: PhantomData,
        }
    }

    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<C>, SwapperError> {
        self.rpc_factory.client_for(chain).map_err(SwapperError::from)
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        get_uniswap_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn is_base_pair(token_in: &Address, token_out: &Address, evm_chain: &EVMChain) -> bool {
        let base_set: HashSet<Address> = HashSet::from_iter(get_base_pair(evm_chain, false).unwrap().path_building_array());
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

    fn parse_request(request: &SwapperQuoteRequest) -> Result<(EVMChain, Address, Address, u128), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::parse_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::parse_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = u128::from_str(&request.value).map_err(SwapperError::from)?;

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
        let deployment = get_uniswap_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let spender = deployment.permit2.to_string();
        check_approval_erc20_with_client(wallet_address.to_string(), token.to_string(), spender, U256::from(amount), client).await
    }

    async fn check_permit2_approval(
        &self,
        client: &JsonRpcClient<C>,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
    ) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let deployment = get_uniswap_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;

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

impl UniswapV4<AlienClient, AlienEvmRpcFactory> {
    pub fn boxed(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
        let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
        Box::new(Self::with_factory(factory))
    }
}

#[async_trait]
impl<C, F> Swapper for UniswapV4<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: EvmRpcClientFactory<C>,
{
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
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
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let client = Arc::new(self.client_for(from_chain)?);

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

        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);
        let calls: Vec<EthereumRpc> = pool_keys
            .iter()
            .map(|pool_key| build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        let batch_call = client.batch_call_requests(calls);
        let mut requests = vec![batch_call];

        let quote_exact_params: Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>>;
        if !Self::is_base_pair(&token_in, &token_out, &evm_chain) {
            let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
            quote_exact_params = build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries);
            for call_array in build_quote_exact_requests(deployment.quoter, &quote_exact_params) {
                requests.push(client.batch_call_requests(call_array));
            }
        } else {
            quote_exact_params = vec![];
        }

        let batch_results = futures::future::join_all(requests).await;

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

        let fee_tier: u32 = fee_tiers[fee_tier_idx % fee_tiers.len()] as u32;
        let asset_id_in = AssetId::from(from_chain, Some(token_in.to_checksum(None)));
        let asset_id_out = AssetId::from(to_chain, Some(token_out.to_checksum(None)));
        let asset_id_intermediary: Option<AssetId> = match batch_idx {
            0 => None,
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
        let client = self.client_for(from_asset.chain)?;
        let wallet_address = eth_address::parse_str(&quote.request.wallet_address)?;
        let (_, token_in, _, amount_in) = Self::parse_request(&quote.request)?;
        self.check_permit2_approval(&client, wallet_address, &token_in.to_checksum(None), U256::from(amount_in), &from_asset.chain)
            .await
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_chain = request.from_asset.chain();
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = self.client_for(from_chain)?;

        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let wallet_address = eth_address::parse_str(&request.wallet_address)?;
        let permit = data.permit2_data().map(|data| data.into());

        let mut gas_limit: Option<String> = None;
        let approval: Option<GemApprovalData> = if quote.request.from_asset.is_native() {
            None
        } else {
            self.check_erc20_approval(&client, wallet_address, &token_in.to_checksum(None), U256::from(amount_in), &from_chain)
                .await?
                .approval_data()
        };
        if approval.is_some() {
            gas_limit = Some(DEFAULT_SWAP_GAS_LIMIT.to_string());
        }

        let sig_deadline = get_sig_deadline();
        let evm_chain = EVMChain::from_chain(from_chain).ok_or(SwapperError::NotSupportedChain)?;
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

        Ok(SwapperQuoteData {
            to: deployment.universal_router.into(),
            value,
            data: HexEncode(encoded),
            approval,
            gas_limit,
        })
    }
}
