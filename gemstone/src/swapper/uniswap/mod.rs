use crate::{
    network::{jsonrpc::batch_into_target, AlienProvider, JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
    swapper::{models::*, slippage::apply_slippage_in_bp, GemSwapProvider, SwapperError},
};
use gem_evm::{
    address::EthereumAddress,
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    permit2::IAllowanceTransfer,
    uniswap::{
        command::{encode_commands, PayPortion, Permit2Permit, Sweep, UniversalRouterCommand, UnwrapWeth, V3SwapExactIn, WrapEth, ADDRESS_THIS},
        contract::IQuoterV2,
        deployment::get_deployment_by_chain,
        FeeTier,
    },
};
use primitives::{AssetId, Chain, ChainType, EVMChain};

use alloy_core::{
    primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U256,
    },
    sol_types::SolCall,
};
use alloy_primitives::aliases::U24;
use async_trait::async_trait;
use serde_json::Value;
use std::{
    fmt::Debug,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

static UNISWAP: &str = "Uniswap";
static DEFAULT_DEADLINE: u64 = 3600;

impl JsonRpcRequest {
    fn from(val: &EthereumRpc, id: u64) -> Self {
        let method = val.method_name();
        let params: Vec<Value> = match val {
            EthereumRpc::GasPrice => vec![],
            EthereumRpc::GetBalance(address) => {
                vec![Value::String(address.to_string())]
            }
            EthereumRpc::Call(tx, block) => {
                let value = serde_json::to_value(tx).unwrap();
                vec![value, block.into()]
            }
        };

        JsonRpcRequest::new(id, method, params)
    }
}

#[derive(Debug)]
pub struct UniswapV3 {}

impl UniswapV3 {
    pub fn new() -> Self {
        Self {}
    }

    pub fn support_chain(&self, chain: Chain) -> bool {
        matches!(
            chain,
            Chain::Ethereum | Chain::Polygon | Chain::AvalancheC | Chain::Arbitrum | Chain::Optimism | Chain::Base | Chain::SmartChain
        )
    }

    fn get_asset_address(asset: &AssetId, evm_chain: EVMChain) -> Result<EthereumAddress, SwapperError> {
        let str = match &asset.token_id {
            Some(token_id) => token_id.to_string(),
            None => evm_chain.weth_contract().unwrap().to_string(),
        };
        EthereumAddress::parse(&str).ok_or(SwapperError::InvalidAddress { address: str })
    }

    fn build_path_with_token(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tier: FeeTier) -> Bytes {
        let mut bytes: Vec<u8> = vec![];
        let fee = U24::from(fee_tier as u32);

        bytes.extend(&token_in.bytes);
        bytes.extend(&fee.to_be_bytes_vec());
        bytes.extend(&token_out.bytes);

        Bytes::from(bytes)
    }

    fn parse_request(request: &SwapQuoteRequest) -> Result<(EVMChain, EthereumAddress, EthereumAddress, U256), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::get_asset_address(&request.from_asset, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset, evm_chain)?;
        let amount_in = U256::from_str(&request.amount).map_err(|_| SwapperError::InvalidAmount)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }

    fn build_quoter_request(request: &SwapQuoteRequest, quoter_v2: &str, amount_in: U256, path: &Bytes) -> EthereumRpc {
        let calldata: Vec<u8> = match request.mode {
            GemSwapMode::ExactIn => {
                let input_call = IQuoterV2::quoteExactInputCall {
                    path: path.clone(),
                    amountIn: amount_in,
                };
                input_call.abi_encode()
            }
            GemSwapMode::ExactOut => {
                let output_call = IQuoterV2::quoteExactOutputCall {
                    path: path.clone(),
                    amountOut: amount_in,
                };
                output_call.abi_encode()
            }
        };

        EthereumRpc::Call(
            TransactionObject::new_call_with_from(&request.wallet_address, quoter_v2, calldata),
            BlockParameter::Latest,
        )
    }

    fn decode_quoter_response(response: &JsonRpcResponse<String>) -> Result<U256, SwapperError> {
        let decoded = HexDecode(&response.result).map_err(|_| SwapperError::NetworkError {
            msg: "Failed to decode hex result".into(),
        })?;
        let quoter_return = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, true)
            .map_err(|err| SwapperError::ABIError { msg: err.to_string() })?
            .amountOut;
        Ok(quoter_return)
    }

    fn build_commands(
        request: &SwapQuoteRequest,
        token_in: &EthereumAddress,
        token_out: &EthereumAddress,
        amount_in: U256,
        quote_amount: U256,
        fee_tier: FeeTier,
        permit: Option<Permit2Permit>,
    ) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
        let options = request.options.clone().unwrap_or_default();
        let fee_options = options.fee.unwrap_or_default();
        let recipient = Address::from_str(&request.wallet_address).map_err(|_| SwapperError::InvalidAddress {
            address: request.wallet_address.clone(),
        })?;

        let mode = request.mode.clone();
        let wrap_input_eth = request.from_asset.is_native();
        let unwrap_output_weth = request.to_asset.is_native();
        let pay_fees = fee_options.bps > 0;

        let path = Self::build_path_with_token(token_in, token_out, fee_tier);

        let mut commands: Vec<UniversalRouterCommand> = vec![];

        match mode {
            GemSwapMode::ExactIn => {
                let amount_out = apply_slippage_in_bp(&quote_amount, options.slippage_bps + fee_options.bps);
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

    async fn check_approval(
        &self,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: Chain,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<ApprovalType, SwapperError> {
        let deployment = get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        // Check token allowance, spender is permit2
        let allowance_data = IERC20::allowanceCall {
            owner: wallet_address,
            spender: Address::parse_checksummed(deployment.permit2, None).unwrap(),
        }
        .abi_encode();
        let allowance_call = EthereumRpc::Call(TransactionObject::new_call(token, allowance_data), BlockParameter::Latest);

        let responses = self.jsonrpc_call(&[allowance_call], provider.clone(), chain).await?;
        let decoded = HexDecode(&responses[0].result).unwrap();
        let allowance = IERC20::allowanceCall::abi_decode_returns(&decoded, false)
            .map_err(|_| SwapperError::ABIError {
                msg: "Invalid erc20 allowance response".into(),
            })?
            ._0;
        if allowance < amount {
            return Ok(ApprovalType::Approve(ApprovalData {
                token: token.to_string(),
                spender: deployment.permit2.to_string(),
                value: amount.to_string(),
            }));
        }

        // Check permit2 allowance, spender is universal router
        let permit2_data = IAllowanceTransfer::allowanceCall {
            _0: wallet_address,
            _1: Address::parse_checksummed(token, None).unwrap(),
            _2: Address::parse_checksummed(deployment.universal_router, None).unwrap(),
        }
        .abi_encode();
        let permit2_call = EthereumRpc::Call(TransactionObject::new_call(deployment.permit2, permit2_data), BlockParameter::Latest);

        let responses = self.jsonrpc_call(&[permit2_call], provider, chain).await?;
        let decoded = HexDecode(&responses[0].result).unwrap();
        let allowance = IAllowanceTransfer::allowanceCall::abi_decode_returns(&decoded, false).map_err(|_| SwapperError::ABIError {
            msg: "Invalid permit2 allowance response".into(),
        })?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
        let expiration: u64 = allowance._2.try_into().unwrap();

        if U256::from(allowance._0) < amount || expiration < timestamp {
            return Ok(ApprovalType::Permit2(ApprovalData {
                token: token.to_string(),
                spender: deployment.universal_router.to_string(),
                value: amount.to_string(),
            }));
        }

        Ok(ApprovalType::None)
    }

    async fn jsonrpc_call(
        &self,
        rpc_calls: &[EthereumRpc],
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<Vec<JsonRpcResponse<String>>, SwapperError> {
        let requests: Vec<JsonRpcRequest> = rpc_calls
            .iter()
            .enumerate()
            .map(|(index, request)| JsonRpcRequest::from(request, index as u64 + 1))
            .collect();

        let endpoint = provider
            .get_endpoint(chain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let targets = vec![batch_into_target(&requests, &endpoint)];

        let data_vec = provider
            .request(targets)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let data = data_vec.first().ok_or(SwapperError::NetworkError { msg: "No result".into() })?;
        let results: Vec<JsonRpcResult<String>> = serde_json::from_slice(data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let responses: Vec<JsonRpcResponse<String>> = results
            .into_iter()
            .filter_map(|result| match result {
                JsonRpcResult::Value(value) => Some(value),
                JsonRpcResult::Error(_) => None,
            })
            .collect();

        if !responses.is_empty() {
            Ok(responses)
        } else {
            Err(SwapperError::NetworkError {
                msg: "All jsonrpc requests failed".into(),
            })
        }
    }
}

#[async_trait]
impl GemSwapProvider for UniswapV3 {
    fn name(&self) -> &'static str {
        UNISWAP
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Prevent swaps on unsupported chains
        if !self.support_chain(request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }

        let wallet_address = Address::parse_checksummed(&request.wallet_address, None).map_err(|_| SwapperError::InvalidAddress {
            address: request.wallet_address.clone(),
        })?;
        let deployment = get_deployment_by_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, token_in, token_out, amount_in) = Self::parse_request(request)?;
        _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;

        // Build path for QuoterV2
        let fee_tiers: Vec<FeeTier> = vec![FeeTier::Lowest, FeeTier::Low, FeeTier::Medium];
        let eth_calls: Vec<EthereumRpc> = fee_tiers
            .iter()
            .map(|fee_tier| {
                let path = Self::build_path_with_token(&token_in, &token_out, fee_tier.clone());
                Self::build_quoter_request(request, deployment.quoter_v2, amount_in, &path)
            })
            .collect();

        let responses = self.jsonrpc_call(&eth_calls, provider.clone(), request.from_asset.chain).await?;

        let mut max_amount_out = U256::from(0);
        let mut fee_tier_idx = 0;
        for response in responses.iter() {
            let amount_out = Self::decode_quoter_response(response)?;
            if amount_out > max_amount_out {
                max_amount_out = amount_out;
                fee_tier_idx = response.id as usize - 1;
            }
        }
        let fee_tier: u32 = fee_tiers[fee_tier_idx].clone() as u32;
        let mut approval_type = ApprovalType::None;
        if !request.from_asset.is_native() {
            // Check allowances
            approval_type = self
                .check_approval(wallet_address, &token_in.to_checksum(), amount_in, request.from_asset.chain, provider)
                .await?;
        }

        Ok(SwapQuote {
            chain_type: ChainType::Ethereum,
            from_value: request.amount.clone(),
            to_value: max_amount_out.to_string(),
            provider: SwapProviderData {
                name: self.name().into(),
                routes: vec![SwapRoute {
                    route_type: String::from("v3-pool"),
                    input: token_in.to_checksum(),
                    output: token_out.to_checksum(),
                    fee_tier: fee_tier.to_string(),
                }],
            },
            approval: approval_type,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(
        &self,
        quote: &SwapQuote,
        _provider: Arc<dyn AlienProvider>,
        permit2: Option<Permit2Data>,
    ) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let (_, token_in, token_out, amount_in) = Self::parse_request(request)?;
        let deployment = get_deployment_by_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let to_amount = U256::from_str(&quote.to_value).map_err(|_| SwapperError::InvalidAmount)?;

        let permit: Option<Permit2Permit> = permit2.map(|x| x.into());
        let fee_tier = FeeTier::try_from(quote.provider.routes[0].fee_tier.as_str()).map_err(|_| SwapperError::InvalidAmount)?;

        let commands = Self::build_commands(request, &token_in, &token_out, amount_in, to_amount, fee_tier, permit)?;
        let deadline = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() + DEFAULT_DEADLINE;
        let encoded = encode_commands(&commands, U256::from(deadline));

        let wrap_input_eth = request.from_asset.is_native();
        let value = if wrap_input_eth { request.amount.clone() } else { String::from("0x0") };
        Ok(SwapQuoteData {
            to: deployment.universal_router.into(),
            value,
            data: HexEncode(encoded),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_core::{hex::decode as HexDecode, hex::encode_prefixed as HexEncode};
    use alloy_primitives::aliases::U256;

    #[test]
    fn test_build_path() {
        // Optimism WETH
        let token0 = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        // USDC
        let token1 = EthereumAddress::parse("0x0b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let bytes = UniswapV3::build_path_with_token(&token0, &token1, FeeTier::Low);

        assert_eq!(
            HexEncode(bytes),
            "0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85"
        )
    }

    #[test]
    fn test_decode_quoter_v2_response() {
        let result = "0x0000000000000000000000000000000000000000000000000000000001884eee000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000014b1e00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000004d04db53840b0aec247bb9bd3ffc00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001";
        let decoded = HexDecode(result).unwrap();
        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, false).unwrap();

        assert_eq!(quote.amountOut, U256::from(25710318));
        assert_eq!(quote.gasEstimate, U256::from(84766));
    }

    #[test]
    fn test_build_commands() {
        let mut request = SwapQuoteRequest {
            // ETH -> USDC
            from_asset: AssetId::from(Chain::Ethereum, None),
            to_asset: AssetId::from(Chain::Ethereum, Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".into())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            amount: "10000000000000000".into(),
            mode: GemSwapMode::ExactIn,
            options: None,
        };

        let token_in = EthereumAddress::parse("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap();
        let token_out = EthereumAddress::parse("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        let amount_in = U256::from(1000000000000000u64);

        // without fee
        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), FeeTier::Low, None).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));

        let options = GemSwapOptions {
            slippage_bps: 100,
            fee: Some(GemSwapFee {
                bps: 25,
                address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            }),
            preferred_providers: vec![],
        };
        request.options = Some(options);

        let commands = UniswapV3::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), FeeTier::Low, None).unwrap();

        assert_eq!(commands.len(), 4);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[3], UniversalRouterCommand::SWEEP(_)));
    }
}
