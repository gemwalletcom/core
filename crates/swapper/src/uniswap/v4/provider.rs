use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use std::{fmt, str::FromStr, sync::Arc};

use crate::{
    FetchQuoteData, Permit2ApprovalData, ProviderData, ProviderType, Quote, QuoteRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::evm::{check_approval_erc20_with_client, check_approval_permit2_with_client},
    approval::get_swap_gas_limit_with_approval,
    eth_address,
    fees::{apply_slippage_in_bp, quote_value_after_reserve_by_chain},
    uniswap::{
        deadline::get_sig_deadline,
        fee_token::{FeeToken, get_fee_token},
        is_native_erc20,
        quote_result::{get_best_quote, get_selected_candidate},
        requires_native_wrapping,
        swap_route::{RouteData, build_swap_route, get_intermediaries},
    },
};
use futures::future::{BoxFuture, join_all};
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{
        FeeTier,
        command::encode_commands,
        deployment::v4::get_uniswap_deployment_by_chain,
        path::{TokenPair, get_base_pair},
    },
};
use gem_jsonrpc::{
    client::JsonRpcClient,
    types::{JsonRpcError, JsonRpcResults},
};
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};

use super::{
    DEFAULT_SWAP_GAS_LIMIT,
    commands::build_commands,
    path::{build_pool_keys, build_quote_exact_params},
    quoter::{build_quote_exact_requests, build_quote_exact_single_request},
};

type QuoteBatchFuture = BoxFuture<'static, Result<JsonRpcResults<String>, JsonRpcError>>;

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

    fn parse_asset_address(asset_id: &str, evm_chain: EVMChain) -> Result<Address, SwapperError> {
        let asset_id = AssetId::new(asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        if requires_native_wrapping(&asset_id) {
            Ok(Address::ZERO)
        } else {
            eth_address::parse_or_weth_address(&asset_id, evm_chain)
        }
    }

    fn parse_request_value(request: &QuoteRequest, value: &str) -> Result<(EVMChain, Address, Address, u128), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::parse_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::parse_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = u128::from_str(value).map_err(SwapperError::from)?;

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
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapperChainAsset::All(*x)).collect()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request_value(request, &from_value)?;
        let fee_tiers = self.get_tiers();
        let base_pair = get_base_pair(&evm_chain, is_native_erc20(from_chain)).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let fee_token_in = FeeToken::new(token_in, request.from_asset.symbol.as_str());
        let fee_token_out = FeeToken::new(token_out, request.to_asset.symbol.as_str());
        let fee_preference = get_fee_token(Some(&base_pair), &fee_token_in, &fee_token_out);
        let fee_bps = request.options.fee.as_ref().map_or(0, |fees| fees.evm.bps);
        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
        };

        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);
        let client = Arc::new(self.client_for(from_chain)?);

        let initial_client = Arc::clone(&client);
        let direct_calls: Vec<EthereumRpc> = pool_keys
            .iter()
            .map(|pool_key| build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        let direct_request: QuoteBatchFuture = Box::pin(async move { initial_client.batch_call_requests(direct_calls).await });
        let direct_batch = (pool_keys.into_iter().map(|pool_key| pool_key.0).collect(), direct_request);

        let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
        let quote_exact_params = build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries);
        let quote_calls = build_quote_exact_requests(deployment.quoter, &quote_exact_params);
        let quote_batches: Vec<(Vec<Vec<TokenPair>>, QuoteBatchFuture)> = std::iter::once(direct_batch)
            .chain(quote_calls.into_iter().zip(quote_exact_params.iter()).map(|(calls, quote_array)| {
                let candidates = quote_array.iter().map(|param| param.0.clone()).collect();
                let client = Arc::clone(&client);
                let request: QuoteBatchFuture = Box::pin(async move { client.batch_call_requests(calls).await });
                (candidates, request)
            }))
            .collect();
        let (quote_candidates, requests): (Vec<Vec<Vec<TokenPair>>>, Vec<QuoteBatchFuture>) = quote_batches.into_iter().unzip();

        let batch_results = join_all(requests).await;

        let quote_result = get_best_quote(&batch_results, super::quoter::decode_quoter_response)?;
        let selected_path = get_selected_candidate(&quote_candidates, &quote_result)?;

        let to_value = if fee_preference.is_input_token {
            quote_result.amount_out
        } else {
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let to_min_value = apply_slippage_in_bp(&to_value, request.options.slippage.bps);

        let routes = build_swap_route(from_chain, selected_path, &to_min_value.to_string())?;

        Ok(Quote {
            from_value: from_value.to_string(),
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes,
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        if requires_native_wrapping(&from_asset) {
            return Ok(None);
        }
        let (_, token_in, _, amount_in) = Self::parse_request_value(&quote.request, &quote.from_value)?;
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

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();
        let (_, token_in, token_out, amount_in) = Self::parse_request_value(request, &quote.from_value)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: RouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let client = self.client_for(from_asset.chain)?;
        let permit = data.permit2_data().map(|data| data.into());
        let wrap_input_eth = requires_native_wrapping(&request.from_asset.asset_id());

        let approval: Option<ApprovalData> = if wrap_input_eth {
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
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_SWAP_GAS_LIMIT);

        let sig_deadline = get_sig_deadline();
        let evm_chain = EVMChain::from_chain(from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = get_base_pair(&evm_chain, is_native_erc20(from_asset.chain));
        let fee_token_in = FeeToken::new(token_in, request.from_asset.symbol.as_str());
        let fee_token_out = FeeToken::new(token_out, request.to_asset.symbol.as_str());
        let fee_preference = get_fee_token(base_pair.as_ref(), &fee_token_in, &fee_token_out);

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

        let value = if wrap_input_eth { quote.from_value.clone() } else { String::from("0") };

        Ok(SwapperQuoteData::new_contract(
            deployment.universal_router.into(),
            value,
            HexEncode(encoded),
            approval,
            gas_limit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::quote_result::QuoteResult;
    use crate::{
        Options, Swapper,
        alien::{AlienError, Target},
        uniswap::default::new_uniswap_v4,
    };
    use alloy_primitives::address;
    use async_trait::async_trait;
    use gem_jsonrpc::{RpcResponse, rpc::RpcProvider as GenericRpcProvider};
    use primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID;
    use serde_json::Value;
    use std::sync::Arc;

    fn quote_exact_single_result(amount_out: u128, gas_estimate: u128) -> String {
        format!("0x{amount_out:064x}{gas_estimate:064x}")
    }

    #[derive(Debug)]
    struct QuoterProviderMock {
        result: String,
        expected_amount: String,
    }

    impl QuoterProviderMock {
        fn new(result: String, expected_amount: u128) -> Arc<Self> {
            Arc::new(Self {
                result,
                expected_amount: format!("{expected_amount:064x}"),
            })
        }

        fn batch_response(&self, target: Target) -> RpcResponse {
            let body = target.body.unwrap();
            let requests: Vec<Value> = serde_json::from_slice(&body).unwrap();
            let matching_amount_count = requests
                .iter()
                .filter(|request| request["params"][0]["data"].as_str().unwrap().contains(&self.expected_amount))
                .count();

            assert!(!requests.is_empty());
            assert_eq!(matching_amount_count, requests.len());

            let responses: Vec<Value> = requests
                .iter()
                .map(|request| {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request["id"].as_u64().unwrap(),
                        "result": self.result,
                    })
                })
                .collect();
            RpcResponse {
                status: Some(200),
                data: serde_json::to_vec(&responses).unwrap(),
            }
        }
    }

    #[async_trait]
    impl GenericRpcProvider for QuoterProviderMock {
        type Error = AlienError;

        async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
            Ok(self.batch_response(target))
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, Self::Error> {
            Ok(String::from("http://localhost:8080"))
        }
    }

    #[tokio::test]
    async fn test_use_max_amount_reserves_native_value_for_quote_and_quote_data() {
        let provider = QuoterProviderMock::new(quote_exact_single_result(25_710_318, 84_766), 1_000_000_000_000_000);
        let swapper = new_uniswap_v4(provider);
        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Ethereum).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "2000000000000000".into(),
            options: Options {
                use_max_amount: true,
                ..Options::default()
            },
        };

        let quote = swapper.get_quote(&request).await.unwrap();
        let quote_data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote.from_value, "1000000000000000");
        assert_eq!(quote_data.value, quote.from_value);
    }

    #[test]
    fn test_selected_candidate_batches_include_direct_routes() {
        let token_in = address!("0x1111111111111111111111111111111111111111");
        let first_intermediary = address!("0x2222222222222222222222222222222222222222");
        let last_intermediary = address!("0x3333333333333333333333333333333333333333");
        let token_out = address!("0x4444444444444444444444444444444444444444");
        let candidates = vec![
            vec![vec![TokenPair {
                token_in,
                token_out,
                fee_tier: FeeTier::Hundred,
            }]],
            vec![TokenPair::new_two_hop_with_fees(
                &token_in,
                &first_intermediary,
                &token_out,
                FeeTier::Hundred,
                FeeTier::ThreeThousand,
            )],
            vec![TokenPair::new_two_hop_with_fees(
                &token_in,
                &last_intermediary,
                &token_out,
                FeeTier::FiveHundred,
                FeeTier::TenThousand,
            )],
        ];

        assert_eq!(
            get_selected_candidate(
                &candidates,
                &QuoteResult {
                    amount_out: U256::from(1),
                    route_idx: 0,
                    batch_idx: 0,
                }
            )
            .unwrap(),
            &candidates[0][0]
        );
        assert_eq!(
            get_selected_candidate(
                &candidates,
                &QuoteResult {
                    amount_out: U256::from(2),
                    route_idx: 0,
                    batch_idx: 1,
                }
            )
            .unwrap(),
            &candidates[1][0]
        );
        assert_eq!(
            get_selected_candidate(
                &candidates,
                &QuoteResult {
                    amount_out: U256::from(3),
                    route_idx: 0,
                    batch_idx: 2,
                }
            )
            .unwrap(),
            &candidates[2][0]
        );
    }

    #[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
    mod swap_integration_tests {
        use crate::{
            FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError,
            fees::{ReferralFee, ReferralFees},
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
                use_max_amount: false,
            };

            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Unichain).into(),
                to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())).into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: "10000000000000000".into(), // 0.01 ETH
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.get_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value.parse::<u64>().unwrap() > 0);

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }
    }
}
