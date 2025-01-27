use super::{
    api::AcrossApi,
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
    DEFAULT_FILL_TIMEOUT,
};
use crate::{
    config::swap_config::SwapReferralFee,
    debug_println,
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20, asset::*, chainlink::ChainlinkPriceFeed, eth_rpc, models::*, slippage::apply_slippage_in_bp, weth_address,
        GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    across::{
        contracts::{
            multicall_handler,
            V3SpokePoolInterface::{self, V3RelayData},
        },
        deployment::{AcrossDeployment, ACROSS_CONFIG_STORE, ACROSS_HUBPOOL},
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    address::EthereumAddress,
    erc20::IERC20,
    jsonrpc::TransactionObject,
    weth::WETH9,
};
use num_bigint::{BigInt, Sign};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    hex::decode as HexDecode,
    hex::encode_prefixed as HexEncode,
    primitives::{Address, Bytes, U256},
    sol_types::{SolCall, SolValue},
};
use async_trait::async_trait;
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug, Default)]
pub struct Across {}

impl Across {
    pub fn boxed() -> Box<dyn GemSwapProvider> {
        Box::new(Self::default())
    }

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        let from = weth_address::normalize_asset(from_asset).unwrap();
        let to = weth_address::normalize_asset(to_asset).unwrap();

        AcrossDeployment::asset_mappings()
            .into_iter()
            .any(|x| x.set.contains(&from) && x.set.contains(&to))
    }

    pub fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    /// Return (message, referral_fee)
    pub fn message_for_multicall_handler(
        &self,
        amount: &U256,
        original_output_asset: &AssetId,
        output_token: &EthereumAddress,
        user_address: &EthereumAddress,
        referral_fee: &SwapReferralFee,
    ) -> (Vec<u8>, U256) {
        if referral_fee.bps == 0 {
            return (vec![], U256::from(0));
        }
        let fee_address = Address::from_str(&referral_fee.address).unwrap();
        let fee_amount = amount * U256::from(referral_fee.bps) / U256::from(10000);
        let user_address = Address::from_slice(&user_address.bytes);
        let user_amount = amount - fee_amount;

        let calls = if original_output_asset.is_native() {
            // output_token is WETH and we need to unwrap it
            Self::unwrap_weth_calls(output_token, amount, &user_address, &user_amount, &fee_address, &fee_amount)
        } else {
            Self::erc20_transfer_calls(output_token, &user_address, &user_amount, &fee_address, &fee_amount)
        };
        let instructions = multicall_handler::Instructions {
            calls,
            fallbackRecipient: user_address,
        };
        let message = instructions.abi_encode();
        (message, fee_amount)
    }

    fn unwrap_weth_calls(
        weth_contract: &EthereumAddress,
        output_amount: &U256,
        user_address: &Address,
        user_amount: &U256,
        fee_address: &Address,
        fee_amount: &U256,
    ) -> Vec<multicall_handler::Call> {
        assert!(fee_amount + user_amount == *output_amount);
        let withdraw_call = WETH9::withdrawCall { wad: *output_amount };
        vec![
            multicall_handler::Call {
                target: Address::from_slice(&weth_contract.bytes),
                callData: withdraw_call.abi_encode().into(),
                value: U256::from(0),
            },
            multicall_handler::Call {
                target: *user_address,
                callData: Bytes::new(),
                value: *user_amount,
            },
            multicall_handler::Call {
                target: *fee_address,
                callData: Bytes::new(),
                value: *fee_amount,
            },
        ]
    }

    fn erc20_transfer_calls(
        token: &EthereumAddress,
        user_address: &Address,
        user_amount: &U256,
        fee_address: &Address,
        fee_amount: &U256,
    ) -> Vec<multicall_handler::Call> {
        let target = Address::from_slice(&token.bytes);
        let user_transfer = IERC20::transferCall {
            to: *user_address,
            value: *user_amount,
        };
        let fee_transfer = IERC20::transferCall {
            to: *fee_address,
            value: *fee_amount,
        };
        vec![
            multicall_handler::Call {
                target,
                callData: user_transfer.abi_encode().into(),
                value: U256::from(0),
            },
            multicall_handler::Call {
                target,
                callData: fee_transfer.abi_encode().into(),
                value: U256::from(0),
            },
        ]
    }

    pub async fn estimate_gas_limit(
        &self,
        amount: &U256,
        is_native: bool,
        input_asset: &AssetId,
        output_token: &EthereumAddress,
        wallet_address: &EthereumAddress,
        message: &[u8],
        deployment: &AcrossDeployment,
        provider: Arc<dyn AlienProvider>,
        chain: &Chain,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain_id: u32 = chain.network_id().parse().unwrap();

        let recipient = if message.is_empty() {
            Address::from_slice(&wallet_address.bytes)
        } else {
            Address::from_str(deployment.multicall_handler().as_str()).unwrap()
        };

        let v3_relay_data = V3RelayData {
            depositor: Address::from_slice(&wallet_address.bytes),
            recipient,
            exclusiveRelayer: Address::ZERO,
            inputToken: Address::from_str(input_asset.token_id.clone().unwrap().as_ref()).unwrap(),
            outputToken: Address::from_slice(&output_token.bytes),
            inputAmount: *amount,
            outputAmount: U256::from(100), // safe amount
            originChainId: U256::from(chain_id),
            depositId: u32::MAX,
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(message.to_vec()),
        };
        let value = if is_native { format!("{:#x}", amount) } else { String::from("0x0") };
        let data = V3SpokePoolInterface::fillV3RelayCall {
            relayData: v3_relay_data.clone(),
            repaymentChainId: U256::from(chain_id),
        }
        .abi_encode();
        let tx = TransactionObject::new_call_to_value(deployment.spoke_pool, &value, data);
        let gas_limit = eth_rpc::estimate_gas(provider, chain, tx).await?;
        Ok((gas_limit, v3_relay_data))
    }

    pub fn update_v3_relay_data(
        &self,
        v3_relay_data: &mut V3RelayData,
        user_address: &EthereumAddress,
        output_amount: &U256,
        original_output_asset: &AssetId,
        output_token: &EthereumAddress,
        timestamp: u32,
        referral_fee: &SwapReferralFee,
    ) -> Result<(), SwapperError> {
        let (message, _) = self.message_for_multicall_handler(output_amount, original_output_asset, output_token, user_address, referral_fee);

        v3_relay_data.outputAmount = *output_amount;
        v3_relay_data.fillDeadline = timestamp + DEFAULT_FILL_TIMEOUT;
        v3_relay_data.message = message.into();

        Ok(())
    }

    pub fn calculate_fee_in_token(fee_in_wei: &U256, token_price: &BigInt, token_decimals: u32) -> U256 {
        let fee = BigInt::from_bytes_le(Sign::Plus, &fee_in_wei.to_le_bytes::<32>());
        let fee_in_token = fee * token_price * BigInt::from(10_u64.pow(token_decimals)) / BigInt::from(10_u64.pow(8)) / BigInt::from(10_u64.pow(18));
        U256::from_le_slice(&fee_in_token.to_bytes_le().1)
    }
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            // WETH
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Base, vec![BASE_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Blast, vec![BLAST_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Linea, vec![LINEA_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::World, vec![WORLD_WETH.id.clone()]),
            // USDC
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone()]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDC.id.clone()]),
            SwapChainAsset::Assets(Chain::Base, vec![BASE_USDC.id.clone()]),
            SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC.id.clone()]),
            // USDT
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::Linea, vec![LINEA_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_USDT.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // does not support same chain swap
        if request.from_asset.chain == request.to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_is_native = request.from_asset.is_native();
        let from_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(|_| SwapperError::InvalidAmount)?;
        let wallet_address = EthereumAddress::parse(&request.wallet_address).ok_or(SwapperError::InvalidAddress {
            address: request.wallet_address.clone(),
        })?;

        let deployment = AcrossDeployment::deployment_by_chain(&request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset, &request.to_asset) {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_asset = weth_address::normalize_asset(&request.from_asset).ok_or(SwapperError::NotSupportedPair)?;
        let output_asset = weth_address::normalize_asset(&request.to_asset.clone()).ok_or(SwapperError::NotSupportedPair)?;
        let output_token = EthereumAddress::parse(&output_asset.clone().token_id.unwrap()).ok_or(SwapperError::InvalidAddress {
            address: format!("{:?}", request.to_asset),
        })?;

        // Get L1 token address
        let mappings = AcrossDeployment::asset_mappings();
        let asset_mapping = mappings.iter().find(|x| x.set.contains(&input_asset)).unwrap();
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).unwrap();
        let mainnet_token = weth_address::parse_into_address(asset_mainnet, from_chain)?;

        let hubpool_client = HubPoolClient {
            contract: ACROSS_HUBPOOL.into(),
            provider: provider.clone(),
            chain: Chain::Ethereum,
        };
        let config_client = ConfigStoreClient {
            contract: ACROSS_CONFIG_STORE.into(),
            provider: provider.clone(),
            chain: Chain::Ethereum,
        };

        let calls = vec![
            hubpool_client.paused_call3(),
            hubpool_client.sync_call3(&mainnet_token),
            hubpool_client.pooled_token_call3(&mainnet_token),
        ];
        let results = eth_rpc::multicall3_call(provider.clone(), &hubpool_client.chain, calls).await?;

        // Check if protocol is paused
        let is_paused = hubpool_client.decoded_paused_call3(&results[0])?;
        if is_paused {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Across protocol is paused".into(),
            });
        }

        // Check bridge amount is too large (Across API has some limit in USD amount but we don't have that info)
        if from_amount > hubpool_client.decoded_pooled_token_call3(&results[2])?.liquidReserves {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Bridge amount is too large".into(),
            });
        }

        // Prepare data for lp fee calculation (token config, utilization, current time)
        let token_config_req = config_client.fetch_config(&mainnet_token); // cache is used inside config_client
        let mut calls = vec![
            hubpool_client.utilization_call3(&mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&mainnet_token, from_amount),
            hubpool_client.get_current_time(),
        ];

        let eth_price_feed = ChainlinkPriceFeed::new_eth_usd_feed(provider.clone());
        if !input_is_native {
            calls.push(eth_price_feed.latest_round_call3());
        }

        let multicall_req = eth_rpc::multicall3_call(provider.clone(), &hubpool_client.chain, calls);

        let batch_results = futures::join!(token_config_req, multicall_req);
        let token_config = batch_results.0.map_err(SwapperError::from)?;
        let multicall_results = batch_results.1.map_err(SwapperError::from)?;

        let util_before = hubpool_client.decoded_utilization_call3(&multicall_results[0])?;
        let util_after = hubpool_client.decoded_utilization_call3(&multicall_results[1])?;
        let timestamp = hubpool_client.decoded_current_time(&multicall_results[2])?;

        let rate_model = Self::get_rate_model(&request.from_asset, &request.to_asset, &token_config);
        let cost_config = &asset_mapping.capital_cost;

        // Calculate lp fee
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent, cost_config.decimals);
        debug_println!("lpfee: {}", lpfee);

        // Calculate relayer fee
        let relayer_calc = RelayerFeeCalculator::default();
        let relayer_fee_percent = relayer_calc.capital_fee_percent(&BigInt::from_str(&request.value).unwrap(), cost_config);
        let relayer_fee = fees::multiply(from_amount, relayer_fee_percent, cost_config.decimals);
        debug_println!("relayer_fee: {}", relayer_fee);

        let referral_config = request.options.fee.clone().unwrap_or_default().evm_bridge;

        // Calculate gas limit / price for relayer
        let remain_amount = from_amount - lpfee - relayer_fee;
        let (message, referral_fee) = self.message_for_multicall_handler(&remain_amount, &request.to_asset, &wallet_address, &output_token, &referral_config);

        let gas_price_req = eth_rpc::fetch_gas_price(provider.clone(), &request.to_asset.chain);
        let gas_limit_req = self.estimate_gas_limit(
            &from_amount,
            input_is_native,
            &input_asset,
            &output_token,
            &wallet_address,
            &message,
            &deployment,
            provider.clone(),
            &request.to_asset.chain,
        );

        let (tuple, gas_price) = futures::join!(gas_limit_req, gas_price_req);
        let (gas_limit, mut v3_relay_data) = tuple?;
        let mut gas_fee = gas_limit * gas_price?;
        if !input_is_native {
            let eth_price = ChainlinkPriceFeed::decoded_answer(&multicall_results[3])?;
            gas_fee = Self::calculate_fee_in_token(&gas_fee, &eth_price, 6);
        }
        debug_println!("gas_fee: {}", gas_fee);

        // Check if bridge amount is too small
        if remain_amount < gas_fee {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Bridge amount is too small".into(),
            });
        }

        let output_amount = remain_amount - gas_fee;
        let output_user_amount = output_amount - referral_fee;

        // Check output amount for user against slippage
        let expect_min = apply_slippage_in_bp(&from_amount, request.options.slippage.bps);
        if output_user_amount < expect_min {
            return Err(SwapperError::ComputeQuoteError {
                msg: format!("Expected amount exceeds slippage, expected: {}, output: {}", expect_min, output_user_amount),
            });
        }

        // Update v3 relay data (was used to estimate gas limit) with final output amount, quote timestamp and referral fee.
        self.update_v3_relay_data(
            &mut v3_relay_data,
            &wallet_address,
            &output_amount,
            &request.to_asset,
            &output_token,
            timestamp,
            &referral_config,
        )?;
        let route_data = HexEncode(v3_relay_data.abi_encode());

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: output_amount.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![SwapRoute {
                    input: input_asset.clone(),
                    output: output_asset.clone(),
                    route_data,
                    gas_estimate: None,
                }],
            },
            request: request.clone(),
        })
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let from_chain = &quote.request.from_asset.chain;
        let deployment = AcrossDeployment::deployment_by_chain(from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let dst_chain_id: u32 = quote.request.to_asset.chain.network_id().parse().unwrap();
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data, true).map_err(|_| SwapperError::InvalidRoute)?;

        let deposit_v3_call = V3SpokePoolInterface::depositV3Call {
            depositor: v3_relay_data.depositor,
            recipient: v3_relay_data.recipient,
            inputToken: v3_relay_data.inputToken,
            outputToken: v3_relay_data.outputToken,
            inputAmount: v3_relay_data.inputAmount,
            outputAmount: v3_relay_data.outputAmount,
            destinationChainId: U256::from(dst_chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: v3_relay_data.fillDeadline - DEFAULT_FILL_TIMEOUT,
            fillDeadline: v3_relay_data.fillDeadline,
            exclusivityDeadline: 0,
            message: v3_relay_data.message,
        }
        .abi_encode();

        let input_is_native = quote.request.from_asset.is_native();
        let value: &str = if input_is_native { &quote.from_value } else { "0" };

        let approval: ApprovalType = {
            if input_is_native {
                ApprovalType::None
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    v3_relay_data.inputToken.to_string(),
                    deployment.spoke_pool.into(),
                    v3_relay_data.inputAmount,
                    provider.clone(),
                    from_chain,
                )
                .await?
            }
        };

        let quote_data = SwapQuoteData {
            to: deployment.spoke_pool.into(),
            value: value.to_string(),
            data: HexEncode(deposit_v3_call.clone()),
            approvals: vec![approval],
        };

        if matches!(data, FetchQuoteData::EstimateGas) {
            let hex_value = format!("{:#x}", U256::from_str(value).unwrap());
            let tx = TransactionObject::new_call_to_value(&quote_data.to, &hex_value, deposit_v3_call);
            let gas_limit = eth_rpc::estimate_gas(provider, from_chain, tx).await?;
            debug_println!("gas_limit: {:?}", gas_limit);
        }

        Ok(quote_data)
    }
    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let api = AcrossApi::new(provider.clone());
        let status = api.deposit_status(&chain, transaction_hash).await?;
        Ok(status.is_filled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::multicall3::IMulticall3;
    use primitives::asset_constants::*;

    #[test]
    fn test_is_supported_pair() {
        let weth_eth: AssetId = WETH_ETH_ASSET_ID.into();
        let weth_op: AssetId = WETH_OP_ASSET_ID.into();
        let weth_arb: AssetId = WETH_ARB_ASSET_ID.into();

        let usdc_eth: AssetId = USDC_ETH_ASSET_ID.into();
        let usdc_arb: AssetId = USDC_ARB_ASSET_ID.into();

        assert!(Across::is_supported_pair(&weth_eth, &weth_op));
        assert!(Across::is_supported_pair(&weth_op, &weth_arb));
        assert!(Across::is_supported_pair(&usdc_eth, &usdc_arb));
        assert!(!Across::is_supported_pair(&weth_eth, &usdc_eth));

        // native asset
        let eth = AssetId::from(Chain::Ethereum, None);
        let op = AssetId::from(Chain::Optimism, None);
        let arb = AssetId::from(Chain::Arbitrum, None);

        assert!(Across::is_supported_pair(&op, &eth));
        assert!(Across::is_supported_pair(&arb, &eth));
        assert!(Across::is_supported_pair(&op, &arb));
    }

    #[test]
    fn test_fee_in_token() {
        let data = HexDecode("0x00000000000000000000000000000000000000000000000700000000000013430000000000000000000000000000000000000000000000000000004e17511aea00000000000000000000000000000000000000000000000000000000677e57a600000000000000000000000000000000000000000000000000000000677e57bb0000000000000000000000000000000000000000000000070000000000001343").unwrap();
        let result = IMulticall3::Result {
            success: true,
            returnData: data.into(),
        };
        let price = ChainlinkPriceFeed::decoded_answer(&result).unwrap();

        assert_eq!(price, BigInt::from(335398640362_u64));

        let gas_fee = U256::from(1861602902696880_u64);
        let fee_in_token = Across::calculate_fee_in_token(&gas_fee, &price, 6);

        assert_eq!(fee_in_token.to_string(), "6243790");
    }
}
