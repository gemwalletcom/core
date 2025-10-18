use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use std::{collections::HashSet, fmt, str::FromStr, sync::Arc, vec};

use crate::{
    FetchQuoteData, Permit2ApprovalData, ProviderData, ProviderType, Quote, QuoteRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::evm::{check_approval_erc20_with_client, check_approval_permit2_with_client},
    eth_address,
    slippage::apply_slippage_in_bp,
    uniswap::{
        deadline::get_sig_deadline,
        fee_token::get_fee_token,
        quote_result::get_best_quote,
        swap_route::{RouteData, build_swap_route, get_intermediaries},
    },
};
use futures::future::{BoxFuture, join_all};
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
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};

use super::{
    DEFAULT_SWAP_GAS_LIMIT,
    commands::build_commands,
    path::{build_pool_keys, build_quote_exact_params},
    quoter::{build_quote_exact_requests, build_quote_exact_single_request},
};

pub struct UniswapV4 {
    pub provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl UniswapV4 {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::UniswapV4),
            rpc_provider,
        }
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        get_uniswap_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<RpcClient>, SwapperError> {
        let endpoint = self.rpc_provider.get_endpoint(chain).map_err(SwapperError::from)?;
        let client = RpcClient::new(endpoint, self.rpc_provider.clone());
        Ok(JsonRpcClient::new(client))
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

    fn parse_request(request: &QuoteRequest) -> Result<(EVMChain, Address, Address, u128), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::parse_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::parse_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = u128::from_str(&request.value).map_err(SwapperError::from)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }
}

impl fmt::Debug for UniswapV4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV4").finish()
    }
}

#[async_trait]
impl Swapper for UniswapV4 {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all()
            .iter()
            .filter(|x| self.support_chain(x))
            .map(|x| SwapperChainAsset::All(*x))
            .collect()
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        let fee_tiers = self.get_tiers();
        let base_pair = get_base_pair(&evm_chain, false).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let fee_preference = get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
        let fee_bps = request.options.clone().fee.unwrap_or_default().evm.bps;
        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
        };

        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);
        let client = Arc::new(self.client_for(from_chain)?);

        let mut requests: Vec<BoxFuture<'static, _>> = Vec::new();
        let initial_client = Arc::clone(&client);
        let direct_calls: Vec<EthereumRpc> = pool_keys
            .iter()
            .map(|pool_key| build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        requests.push(Box::pin(async move { initial_client.batch_call_requests(direct_calls).await }));

        let quote_exact_params: Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>>;
        if !Self::is_base_pair(&token_in, &token_out, &evm_chain) {
            let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
            quote_exact_params = build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries);
            build_quote_exact_requests(deployment.quoter, &quote_exact_params)
                .iter()
                .for_each(|call_array| {
                    let client = Arc::clone(&client);
                    let calls = call_array.clone();
                    requests.push(Box::pin(async move { client.batch_call_requests(calls).await }));
                });
        } else {
            quote_exact_params = vec![];
        }

        let batch_results = join_all(requests).await;

        let quote_result = get_best_quote(&batch_results, super::quoter::decode_quoter_response)?;

        let fee_tier_idx = quote_result.fee_tier_idx;
        let batch_idx = quote_result.batch_idx;
        let gas_estimate = quote_result.gas_estimate;

        let to_value = if fee_preference.is_input_token {
            quote_result.amount_out
        } else {
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

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: routes.clone(),
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn fetch_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        if from_asset.is_native() {
            return Ok(None);
        }
        let (_, token_in, _, amount_in) = Self::parse_request(&quote.request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = self.client_for(from_asset.chain)?;
        let permit2_data = check_approval_permit2_with_client(
            deployment.permit2,
            quote.request.wallet_address.clone(),
            token_in.to_string(),
            deployment.universal_router.to_string(),
            U256::from(amount_in),
            &client,
        )
        .await?
        .permit2_data();

        Ok(permit2_data)
    }

    async fn fetch_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let client = self.client_for(from_asset.chain)?;
        let permit = data.permit2_data().map(|data| data.into());

        let mut gas_limit: Option<String> = None;
        let approval: Option<ApprovalData> = if quote.request.from_asset.is_native() {
            None
        } else {
            check_approval_erc20_with_client(
                request.wallet_address.clone(),
                token_in.to_string(),
                deployment.permit2.to_string(),
                U256::from(amount_in),
                &client,
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

        Ok(SwapperQuoteData {
            to: deployment.universal_router.into(),
            value,
            data: HexEncode(encoded),
            memo: None,
            approval,
            gas_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperMode, alien::mock::ProviderMock};
    use std::sync::Arc;

    #[test]
    fn test_is_base_pair() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = UniswapV4::new(provider);
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::SmartChain, Some("0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string())).into(),
            to_asset: AssetId::from_chain(Chain::SmartChain).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "40000000000000000".into(), // 0.04 Cake
            mode: SwapperMode::ExactIn,
            options: Options::default(),
        };

        let (evm_chain, token_in, token_out, _) = UniswapV4::parse_request(&request).unwrap();

        assert!(UniswapV4::is_base_pair(&token_in, &token_out, &evm_chain));
        // Ensure provider field is used to avoid warnings
        assert_eq!(swapper.provider.id, SwapperProvider::UniswapV4);
    }

    #[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
    mod swap_integration_tests {
        use super::*;
        use crate::{
            FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError, SwapperMode, SwapperProvider,
            config::{ReferralFee, ReferralFees},
            uniswap,
        };
        use primitives::{AssetId, Chain};
        use std::{sync::Arc, time::SystemTime};

        #[tokio::test]
        async fn test_v4_quoter() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                })),
                preferred_providers: vec![SwapperProvider::UniswapV4],
                use_max_amount: false,
            };

            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Unichain).into(),
                to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())).into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: "10000000000000000".into(), // 0.01 ETH
                mode: SwapperMode::ExactIn,
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.fetch_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value.parse::<u64>().unwrap() > 0);

            let quote_data = swap_provider.fetch_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }
    }
}
