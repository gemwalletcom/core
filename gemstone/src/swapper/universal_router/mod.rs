mod pancakeswap_router;
mod uniswap_router;

use crate::{
    network::{jsonrpc::batch_jsonrpc_call, AlienProvider, JsonRpcResponse, JsonRpcResult},
    swapper::{
        approval::{check_approval, CheckApprovalType},
        models::*,
        slippage::apply_slippage_in_bp,
        GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::{
        command::{encode_commands, PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand, UnwrapWeth, V3SwapExactIn, WrapEth, ADDRESS_THIS},
        contract::IQuoterV2,
        deployment::V3Deployment,
        path::{build_direct_pair, build_pairs, get_base_pair, BasePair, TokenPair},
        FeeTier,
    },
};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U256,
    },
    sol_types::SolCall,
};
use async_trait::async_trait;
use std::{
    fmt::Debug,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use super::weth_address;

static DEFAULT_DEADLINE: u64 = 3600; // 1 hour

pub struct FeePreference {
    pub fee_token: EthereumAddress,
    pub is_input_token: bool,
}

pub trait UniversalRouterProvider: Send + Sync + Debug {
    fn provider(&self) -> SwapProvider;
    fn get_tiers(&self) -> Vec<FeeTier>;
    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment>;
}

#[derive(Debug)]
pub struct UniswapV3 {
    provider: Box<dyn UniversalRouterProvider>,
}

impl UniswapV3 {
    pub fn new(provider: Box<dyn UniversalRouterProvider>) -> Self {
        Self { provider }
    }

    pub fn new_uniswap() -> Self {
        Self::new(Box::new(uniswap_router::UniswapUniversalRouter {}))
    }
    pub fn new_pancakeswap() -> Self {
        Self::new(Box::new(pancakeswap_router::PancakeSwapUniversalRouter {}))
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

    fn get_intermediaries(token_in: &EthereumAddress, token_out: &EthereumAddress, base_pair: &BasePair) -> Vec<EthereumAddress> {
        base_pair
            .to_array()
            .iter()
            .filter(|intermediary| *intermediary != token_in && *intermediary != token_out)
            .cloned()
            .collect()
    }

    // Return (fee token, is_input_token)
    fn get_fee_token(mode: &GemSwapMode, base_pair: Option<&BasePair>, input: &EthereumAddress, output: &EthereumAddress) -> FeePreference {
        let use_input_as_fee_token = match mode {
            GemSwapMode::ExactIn => {
                if let Some(pair) = base_pair {
                    let set = pair.to_set();
                    set.contains(input) && !set.contains(output)
                } else {
                    false
                }
            }
            GemSwapMode::ExactOut => true,
        };
        FeePreference {
            fee_token: if use_input_as_fee_token { input.clone() } else { output.clone() },
            is_input_token: use_input_as_fee_token,
        }
    }

    fn build_paths(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tiers: &[FeeTier], base_pair: &BasePair) -> Vec<Vec<(Vec<TokenPair>, Bytes)>> {
        let mut paths: Vec<Vec<(Vec<TokenPair>, Bytes)>> = vec![];
        let direct_paths: Vec<_> = fee_tiers
            .iter()
            .map(|fee_tier| {
                (
                    vec![TokenPair {
                        token_in: token_in.clone(),
                        token_out: token_out.clone(),
                        fee_tier: fee_tier.clone(),
                    }],
                    build_direct_pair(token_in, token_out, fee_tier.clone() as u32),
                )
            })
            .collect();
        paths.push(direct_paths);

        let intermediaries = Self::get_intermediaries(token_in, token_out, base_pair);
        intermediaries.iter().for_each(|intermediary| {
            let token_pairs: Vec<Vec<TokenPair>> = fee_tiers
                .iter()
                .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, fee_tier))
                .collect();
            let pair_paths: Vec<_> = token_pairs.iter().map(|token_pairs| (token_pairs.to_vec(), build_pairs(token_pairs))).collect();
            paths.push(pair_paths);
        });
        paths
    }

    fn build_paths_with_routes(routes: &[SwapRoute]) -> Result<Bytes, SwapperError> {
        if routes.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        let fee_tier = FeeTier::try_from(routes[0].route_data.as_str()).map_err(|_| SwapperError::InvalidAmount)?;
        let token_pairs: Vec<TokenPair> = routes
            .iter()
            .map(|route| TokenPair {
                token_in: route.input.clone().into(),
                token_out: route.output.clone().into(),
                fee_tier: fee_tier.clone(),
            })
            .collect();
        let paths = build_pairs(&token_pairs);
        Ok(paths)
    }

    fn build_quoter_request(mode: &GemSwapMode, wallet_address: &str, quoter_v2: &str, amount_in: U256, path: &Bytes) -> EthereumRpc {
        let call_data: Vec<u8> = match mode {
            GemSwapMode::ExactIn => IQuoterV2::quoteExactInputCall {
                path: path.clone(),
                amountIn: amount_in,
            }
            .abi_encode(),
            GemSwapMode::ExactOut => IQuoterV2::quoteExactOutputCall {
                path: path.clone(),
                amountOut: amount_in,
            }
            .abi_encode(),
        };

        EthereumRpc::Call(
            TransactionObject::new_call_with_from(wallet_address, quoter_v2, call_data),
            BlockParameter::Latest,
        )
    }

    fn build_swap_route(
        token_in: &AssetId,
        intermediary: Option<&AssetId>,
        token_out: &AssetId,
        fee_tier: &str,
        gas_estimate: Option<String>,
    ) -> Vec<SwapRoute> {
        if let Some(intermediary) = intermediary {
            vec![
                SwapRoute {
                    input: token_in.clone(),
                    output: intermediary.clone(),
                    route_data: fee_tier.to_string(),
                    gas_estimate: gas_estimate.clone(),
                },
                SwapRoute {
                    input: intermediary.clone(),
                    output: token_out.clone(),
                    route_data: fee_tier.to_string(),
                    gas_estimate: None,
                },
            ]
        } else {
            vec![SwapRoute {
                input: token_in.clone(),
                output: token_out.clone(),
                route_data: fee_tier.to_string(),
                gas_estimate: gas_estimate.clone(),
            }]
        }
    }

    // Returns (amountOut, gasEstimate)
    fn decode_quoter_response(response: &JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError> {
        let decoded = HexDecode(&response.result).map_err(|_| SwapperError::NetworkError {
            msg: "Failed to decode hex result".into(),
        })?;
        let quoter_return =
            IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, true).map_err(|err| SwapperError::ABIError { msg: err.to_string() })?;

        Ok((quoter_return.amountOut, quoter_return.gasEstimate))
    }

    fn build_commands(
        request: &SwapQuoteRequest,
        token_in: &EthereumAddress,
        token_out: &EthereumAddress,
        amount_in: U256,
        quote_amount: U256,
        path: &Bytes,
        permit: Option<Permit2Permit>,
        fee_token_is_input: bool,
    ) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
        let options = request.options.clone();
        let fee_options = options.fee.unwrap_or_default().evm;
        let recipient = Address::from_str(&request.wallet_address).map_err(|_| SwapperError::InvalidAddress {
            address: request.wallet_address.clone(),
        })?;

        let mode = request.mode.clone();
        let wrap_input_eth = request.from_asset.is_native();
        let unwrap_output_weth = request.to_asset.is_native();
        let pay_fees = fee_options.bps > 0;

        let mut commands: Vec<UniversalRouterCommand> = vec![];

        match mode {
            GemSwapMode::ExactIn => {
                let amount_out = apply_slippage_in_bp(&quote_amount, options.slippage.bps + fee_options.bps);
                if wrap_input_eth {
                    // Wrap ETH, recipient is this_address
                    commands.push(UniversalRouterCommand::WRAP_ETH(WrapEth {
                        recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                        amount_min: amount_in,
                    }));
                } else if let Some(permit) = permit {
                    commands.push(UniversalRouterCommand::PERMIT2_PERMIT(permit));
                }

                // payer_is_user: is true when swapping tokens
                let payer_is_user = !wrap_input_eth;
                if pay_fees {
                    if fee_token_is_input {
                        // insert TRANSFER fee first
                        let fee = amount_in * U256::from(fee_options.bps) / U256::from(10000);
                        let fee_recipient = Address::from_str(fee_options.address.as_str()).unwrap();
                        if wrap_input_eth {
                            // if input is native ETH, we can transfer directly because of WRAP_ETH command
                            commands.push(UniversalRouterCommand::TRANSFER(Transfer {
                                token: Address::from_slice(&token_in.bytes),
                                recipient: fee_recipient,
                                value: fee,
                            }));
                        } else {
                            // call permit2 transfer instead
                            commands.push(UniversalRouterCommand::PERMIT2_TRANSFER_FROM(Transfer {
                                token: Address::from_slice(&token_in.bytes),
                                recipient: fee_recipient,
                                value: fee,
                            }));
                        };

                        // insert V3_SWAP_EXACT_IN with amount - fee, recipient is user address
                        commands.push(UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                            recipient,
                            amount_in: amount_in - fee,
                            amount_out_min: amount_out,
                            path: path.clone(),
                            payer_is_user,
                        }));
                    } else {
                        // insert V3_SWAP_EXACT_IN
                        // amount_out_min: if needs to pay fees, amount_out_min set to 0 and we will sweep the rest
                        commands.push(UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                            recipient: Address::from_str(ADDRESS_THIS).unwrap(),
                            amount_in,
                            amount_out_min: if pay_fees { U256::from(0) } else { amount_out },
                            path: path.clone(),
                            payer_is_user,
                        }));

                        // insert PAY_PORTION to fee_address
                        commands.push(UniversalRouterCommand::PAY_PORTION(PayPortion {
                            token: Address::from_slice(&token_out.bytes),
                            recipient: Address::from_str(fee_options.address.as_str()).unwrap(),
                            bips: U256::from(fee_options.bps),
                        }));

                        if !unwrap_output_weth {
                            // MSG_SENDER should be the address of the caller
                            commands.push(UniversalRouterCommand::SWEEP(Sweep {
                                token: Address::from_slice(&token_out.bytes),
                                recipient,
                                amount_min: U256::from(amount_out),
                            }));
                        }
                    }
                } else {
                    // insert V3_SWAP_EXACT_IN
                    commands.push(UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                        recipient,
                        amount_in,
                        amount_out_min: amount_out,
                        path: path.clone(),
                        payer_is_user,
                    }));
                }

                if unwrap_output_weth {
                    // insert UNWRAP_WETH
                    commands.push(UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
                        recipient,
                        amount_min: U256::from(amount_out),
                    }));
                }
            }
            GemSwapMode::ExactOut => {
                todo!("swap exact out not implemented");
            }
        }
        Ok(commands)
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
        let erc20_type = CheckApprovalType::ERC20(wallet_address.to_string(), token.to_string(), deployment.permit2.to_string(), amount);

        check_approval(erc20_type, provider.clone(), chain).await
    }

    async fn check_permit2_approval(
        &self,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<ApprovalType, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let permit2_type = CheckApprovalType::Permit2(
            deployment.permit2.to_string(),
            wallet_address.to_string(),
            token.to_string(),
            deployment.universal_router.to_string(),
            amount,
        );

        check_approval(permit2_type, provider.clone(), chain).await
    }
}

#[async_trait]
impl GemSwapProvider for UniswapV3 {
    fn provider(&self) -> SwapProvider {
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
        let base_pair = get_base_pair(&evm_chain).ok_or(SwapperError::ComputeQuoteError {
            msg: "base pair not found".into(),
        })?;

        let fee_preference = Self::get_fee_token(&request.mode, Some(&base_pair), &token_in, &token_out);
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
        let paths_array = Self::build_paths(&token_in, &token_out, &fee_tiers, &base_pair);
        let requests: Vec<_> = paths_array
            .iter()
            .map(|paths| {
                let calls: Vec<EthereumRpc> = paths
                    .iter()
                    .map(|path| Self::build_quoter_request(&request.mode, &request.wallet_address, deployment.quoter_v2, quote_amount_in, &path.1))
                    .collect();

                // batch fee_tiers.len() requests into one jsonrpc call
                batch_jsonrpc_call(calls, provider.clone(), &request.from_asset.chain)
            })
            .collect();

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
                                let quoter_tuple = Self::decode_quoter_response(value)?;
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
                let first_token_out = &paths_array[batch_idx][0].0[0].token_out;
                Some(AssetId::from(request.to_asset.chain, Some(first_token_out.to_checksum())))
            }
        };
        let routes = Self::build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &fee_tier.to_string(), gas_estimate);

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

    async fn fetch_permit2_for_quote(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>) -> Result<ApprovalType, SwapperError> {
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

        let permit = match data {
            FetchQuoteData::Permit2(data) => Some(data.into()),
            _ => None,
        };

        let approval = {
            if quote.request.from_asset.is_native() {
                ApprovalType::None
            } else {
                // Check if need to approve permit2 contract
                self.check_erc20_approval(wallet_address, &token_in.to_checksum(), amount_in, &request.from_asset.chain, provider)
                    .await?
            }
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
        let sig_deadline = now + DEFAULT_DEADLINE;

        let evm_chain = EVMChain::from_chain(quote.request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = get_base_pair(&evm_chain);
        let fee_preference = Self::get_fee_token(&request.mode, base_pair.as_ref(), &token_in, &token_out);

        let path: Bytes = Self::build_paths_with_routes(&quote.data.routes)?;
        let commands = Self::build_commands(
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
        })
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        // TODO: the transaction status from the RPC
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::swap_config::{SwapReferralFee, SwapReferralFees},
        swapper::permit2_data::*,
    };
    use alloy_core::hex::decode as HexDecode;
    use alloy_primitives::aliases::U256;

    #[test]
    fn test_decode_quoter_v2_response() {
        let result = "0x0000000000000000000000000000000000000000000000000000000001884eee000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000014b1e00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000004d04db53840b0aec247bb9bd3ffc00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001";
        let decoded = HexDecode(result).unwrap();
        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, false).unwrap();

        assert_eq!(quote.amountOut, U256::from(25710318));
        assert_eq!(quote.gasEstimate, U256::from(84766));
    }

    #[test]
    fn test_build_commands_eth_to_token() {
        let mut request = SwapQuoteRequest {
            // ETH -> USDC
            from_asset: AssetId::from(Chain::Ethereum, None),
            to_asset: AssetId::from(Chain::Ethereum, Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".into())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000000000000".into(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions::default(),
        };

        let token_in = EthereumAddress::parse("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap();
        let token_out = EthereumAddress::parse("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        let amount_in = U256::from(1000000000000000u64);

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred as u32);
        // without fee
        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), &path, None, false).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));

        let options = GemSwapOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
            })),
            preferred_providers: vec![],
        };
        request.options = options;

        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), &path, None, false).unwrap();

        assert_eq!(commands.len(), 4);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[3], UniversalRouterCommand::SWEEP(_)));
    }

    #[test]
    fn test_build_commands_usdc_to_usdt() {
        let request = SwapQuoteRequest {
            // USDC -> USDT
            from_asset: AssetId::from(Chain::Optimism, Some("0x0b2c639c533813f4aa9d7837caf62653d097ff85".into())),
            to_asset: AssetId::from(Chain::Optimism, Some("0x94b008aa00579c1307b0ef2c499ad98a8ce58e58".into())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "6500000".into(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions::default(),
        };

        let token_in = EthereumAddress::parse(request.from_asset.token_id.as_ref().unwrap()).unwrap();
        let token_out = EthereumAddress::parse(request.to_asset.token_id.as_ref().unwrap()).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667593,
                    nonce: 0,
                },
                spender: "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8".into(),
                sig_deadline: 1730077393,
            },
            signature: hex::decode(
                "8f32d2e66506a4f424b1b23309ed75d338534d0912129a8aa3381fab4eb8032f160e0988f10f512b19a58c2a689416366c61cc0c483c3b5322dc91f8b60107671b",
            )
            .unwrap(),
        };

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred as u32);
        let commands = UniswapV3::build_commands(
            &request,
            &token_in,
            &token_out,
            amount_in,
            U256::from(6507936),
            &path,
            Some(permit2_data.into()),
            false,
        )
        .unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_PERMIT(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_build_commands_usdc_to_aave() {
        let request = SwapQuoteRequest {
            // USDC -> AAVE
            from_asset: AssetId::from(Chain::Optimism, Some("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into())),
            to_asset: AssetId::from(Chain::Optimism, Some("0x76fb31fb4af56892a25e32cfc43de717950c9278".into())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "5064985".into(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions {
                slippage: 100.into(),
                fee: Some(SwapReferralFees::evm(SwapReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                preferred_providers: vec![],
            },
        };

        let token_in = EthereumAddress::parse(request.from_asset.token_id.as_ref().unwrap()).unwrap();
        let token_out = EthereumAddress::parse(request.to_asset.token_id.as_ref().unwrap()).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred as u32);
        // fee token is output token
        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(33377662359182269u64), &path, None, false).unwrap();

        assert_eq!(commands.len(), 3);

        assert!(matches!(commands[0], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));

        // fee token is input token
        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(33377662359182269u64), &path, None, true).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_TRANSFER_FROM(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_build_commands_usdce_to_eth() {
        let request = SwapQuoteRequest {
            // USDCE -> ETH
            from_asset: AssetId::from(Chain::Optimism, Some("0x7F5c764cBc14f9669B88837ca1490cCa17c31607".into())),
            to_asset: AssetId::from(Chain::Ethereum, None),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000".into(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions {
                slippage: 100.into(),
                fee: Some(SwapReferralFees::evm(SwapReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                preferred_providers: vec![],
            },
        };

        let token_in = EthereumAddress::parse(request.from_asset.token_id.as_ref().unwrap()).unwrap();
        let token_out = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: request.from_asset.token_id.clone().unwrap(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667502,
                    nonce: 0,
                },
                spender: "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8".into(),
                sig_deadline: 1730077302,
            },
            signature: hex::decode(
                "00e96ed0f5bf5cca62dc9d9753960d83c8be83224456559a1e93a66d972a019f6f328a470f8257d3950b4cb7cd0024d789b4fcd9e80c4eb43d82a38d9e5332f31b",
            )
            .unwrap(),
        };

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred as u32);
        let commands = UniswapV3::build_commands(
            &request,
            &token_in,
            &token_out,
            amount_in,
            U256::from(3997001989341576u64),
            &path,
            Some(permit2_data.into()),
            false,
        )
        .unwrap();

        assert_eq!(commands.len(), 4);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_PERMIT(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[3], UniversalRouterCommand::UNWRAP_WETH(_)));
    }

    #[test]
    fn test_build_commands_eth_to_uni_with_input_fee() {
        // Replicate https://optimistic.etherscan.io/tx/0x18277deea3e273a7fb9abc985269dcdabe3d34c2b604fbd82dcd0a5a5204f72c
        let request = SwapQuoteRequest {
            // ETH -> UNI
            from_asset: AssetId::from(Chain::Optimism, None),
            to_asset: AssetId::from(Chain::Optimism, Some("0x6fd9d7ad17242c41f7131d257212c54a0e816691".into())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "1000000000000000".into(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions {
                slippage: 100.into(),
                fee: Some(SwapReferralFees::evm(SwapReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                preferred_providers: vec![],
            },
        };

        let token_in = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        let token_out = EthereumAddress::parse(&request.to_asset.token_id.clone().unwrap()).unwrap();
        let amount_in = U256::from_str(request.value.as_str()).unwrap();

        let path = build_direct_pair(&token_in, &token_out, FeeTier::ThreeThousand as u32);
        let commands = UniswapV3::build_commands(
            &request,
            &token_in,
            &token_out,
            amount_in,
            U256::from(244440440678888410_u64),
            &path,
            None,
            true,
        )
        .unwrap();

        assert_eq!(commands.len(), 3);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::TRANSFER(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_get_fee_token() {
        let evm_chain = EVMChain::Ethereum;
        let mode = GemSwapMode::ExactIn;
        let base_pair = get_base_pair(&evm_chain);

        let weth = EthereumAddress::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        let uni = EthereumAddress::from_str("0x1f9840a85d5af5bf1d1762f925bdaddc4201f984").unwrap();
        let usdc = EthereumAddress::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();

        // WETH -> UNI (fee_token is WETH)
        let fee_preference = UniswapV3::get_fee_token(&mode, base_pair.as_ref(), &weth, &uni);

        assert_eq!(fee_preference.fee_token, weth);
        assert!(fee_preference.is_input_token);

        // WETH -> USDC (fee_token is USDC)
        let fee_preference = UniswapV3::get_fee_token(&mode, base_pair.as_ref(), &weth, &usdc);
        assert_eq!(fee_preference.fee_token, usdc);
        assert!(!fee_preference.is_input_token);

        // USDC -> WETH (fee_token is WETH)
        let fee_preference = UniswapV3::get_fee_token(&mode, base_pair.as_ref(), &usdc, &weth);

        assert_eq!(fee_preference.fee_token, weth);
        assert!(!fee_preference.is_input_token);

        // USDC -> UNI (fee_token is USDC)
        let fee_preference = UniswapV3::get_fee_token(&mode, base_pair.as_ref(), &usdc, &uni);

        assert_eq!(fee_preference.fee_token, usdc);
        assert!(fee_preference.is_input_token);
    }
}
