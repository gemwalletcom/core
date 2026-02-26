use super::{
    DEFAULT_FILL_TIMEOUT,
    api::{ParsedDeposit, filled_relay_topic, parse_deposit_from_logs},
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
};
use crate::{
    SwapResult, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    across::{DEFAULT_DEPOSIT_GAS_LIMIT, DEFAULT_FILL_GAS_LIMIT},
    alien::RpcProvider,
    approval::check_approval_erc20,
    asset::*,
    chainlink::ChainlinkPriceFeed,
    client_factory::create_eth_client,
    config::ReferralFee,
    eth_address,
    models::*,
};
use alloy_primitives::{
    Address, Bytes, U256,
    hex::{decode as HexDecode, encode_prefixed as HexEncode},
};
use alloy_sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use gem_evm::{
    across::{
        contracts::{
            V3SpokePoolInterface::{self, V3RelayData},
            multicall_handler,
        },
        deployment::AcrossDeployment,
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    contracts::erc20::IERC20,
    jsonrpc::TransactionObject,
    multicall3::IMulticall3,
    weth::WETH9,
};
use num_bigint::{BigInt, Sign};
use primitives::{AssetId, Chain, EVMChain, TransactionSwapMetadata, swap::ApprovalData};
use serde_serializers::biguint_from_hex_str;
use std::{fmt::Debug, str::FromStr, sync::Arc};

fn resolve_token_asset(chain: Chain, token_address: &str) -> Option<AssetId> {
    let evm_chain = EVMChain::from_chain(chain)?;
    let address = gem_evm::ethereum_address_checksum(token_address).ok()?;
    if evm_chain.weth_contract().is_some_and(|w| w == address) {
        return Some(AssetId::from_chain(chain));
    }
    Some(AssetId::from_token(chain, &address))
}

pub struct AcrossCrossChain;

impl crate::cross_chain::CrossChainProvider for AcrossCrossChain {
    fn provider(&self) -> SwapperProvider {
        SwapperProvider::Across
    }

    fn is_swap(&self, transaction: &primitives::Transaction) -> bool {
        AcrossDeployment::deployment_by_chain(&transaction.asset_id.chain).is_some_and(|d| d.spoke_pool == transaction.to)
    }
}

#[derive(Debug)]
pub struct Across {
    pub provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Across {
    fn bigint_to_u256(value: &BigInt) -> Result<U256, SwapperError> {
        if value.sign() == Sign::Minus {
            return Err(SwapperError::ComputeQuoteError("Negative value provided for gas computation".into()));
        }

        let bytes = value.to_bytes_be().1;
        Ok(U256::from_be_slice(bytes.as_slice()))
    }

    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Across),
            rpc_provider,
        }
    }

    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
        Box::new(Self::new(rpc_provider))
    }

    fn build_swap_metadata(deposit: &ParsedDeposit, destination_chain_id: u64) -> Option<TransactionSwapMetadata> {
        let origin_chain = Chain::from_chain_id(deposit.origin_chain_id)?;
        let from_asset = resolve_token_asset(origin_chain, deposit.input_token.as_ref()?)?;
        let to_chain = Chain::from_chain_id(destination_chain_id)?;
        let to_asset = resolve_token_asset(to_chain, deposit.output_token.as_ref()?)?;
        Some(TransactionSwapMetadata {
            from_asset,
            from_value: deposit.input_amount.clone()?,
            to_asset,
            to_value: deposit.output_amount.clone()?,
            provider: Some(SwapperProvider::Across.as_ref().to_string()),
        })
    }

    async fn check_fill_on_chain(&self, origin_chain_id: u64, deposit_id: u64, destination_chain: Chain) -> Result<Option<String>, SwapperError> {
        let deployment = AcrossDeployment::deployment_by_chain(&destination_chain).ok_or(SwapperError::NotSupportedChain)?;
        let client = create_eth_client(self.rpc_provider.clone(), destination_chain)?;

        let topic0 = filled_relay_topic();
        let topic1 = format!("{:#066x}", U256::from(origin_chain_id));
        let topic2 = format!("{:#066x}", U256::from(deposit_id));
        let topics = vec![Some(topic0), Some(topic1), Some(topic2)];

        let logs = client.get_logs(deployment.spoke_pool, &topics, "0x0", "latest").await.map_err(SwapperError::from)?;

        Ok(logs.first().and_then(|l| l.transaction_hash.clone()))
    }

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        let Some(from) = eth_address::convert_native_to_weth(from_asset) else {
            return false;
        };
        let Some(to) = eth_address::convert_native_to_weth(to_asset) else {
            return false;
        };

        AcrossDeployment::asset_mappings().into_iter().any(|x| x.set.contains(&from) && x.set.contains(&to))
    }

    pub fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    async fn gas_price(&self, chain: Chain) -> Result<U256, SwapperError> {
        let gas_price = create_eth_client(self.rpc_provider.clone(), chain)?.gas_price().await?;
        Self::bigint_to_u256(&gas_price)
    }

    async fn multicall3(&self, chain: Chain, calls: Vec<IMulticall3::Call3>) -> Result<Vec<IMulticall3::Result>, SwapperError> {
        create_eth_client(self.rpc_provider.clone(), chain)?
            .multicall3(calls)
            .await
            .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))
    }

    async fn estimate_gas_transaction(&self, chain: Chain, tx: TransactionObject) -> Result<U256, SwapperError> {
        let client = create_eth_client(self.rpc_provider.clone(), chain)?;
        let gas_hex = client.estimate_gas(tx.from.as_deref(), &tx.to, tx.value.as_deref(), Some(tx.data.as_str())).await?;

        let gas_biguint = biguint_from_hex_str(&gas_hex).map_err(|e| SwapperError::ComputeQuoteError(format!("Failed to parse gas estimate: {e}")))?;
        let gas_bigint = BigInt::from_biguint(Sign::Plus, gas_biguint);
        Self::bigint_to_u256(&gas_bigint)
    }

    /// Return (message, referral_fee)
    pub fn message_for_multicall_handler(
        &self,
        amount: &U256,
        original_output_asset: &AssetId,
        output_token: &Address,
        user_address: &Address,
        referral_fee: &ReferralFee,
    ) -> (Vec<u8>, U256) {
        if referral_fee.bps == 0 {
            return (vec![], U256::from(0));
        }
        let fee_address = Address::from_str(&referral_fee.address).unwrap();
        let fee_amount = amount * U256::from(referral_fee.bps) / U256::from(10000);
        let user_amount = amount - fee_amount;

        let calls = if original_output_asset.is_native() {
            // output_token is WETH and we need to unwrap it
            Self::unwrap_weth_calls(output_token, amount, user_address, &user_amount, &fee_address, &fee_amount)
        } else {
            Self::erc20_transfer_calls(output_token, user_address, &user_amount, &fee_address, &fee_amount)
        };
        let instructions = multicall_handler::Instructions {
            calls,
            fallbackRecipient: *user_address,
        };
        let message = instructions.abi_encode();
        (message, fee_amount)
    }

    fn unwrap_weth_calls(
        weth_contract: &Address,
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
                target: *weth_contract,
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

    fn erc20_transfer_calls(token: &Address, user_address: &Address, user_amount: &U256, fee_address: &Address, fee_amount: &U256) -> Vec<multicall_handler::Call> {
        let target = *token;
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
        output_token: &Address,
        wallet_address: &Address,
        message: &[u8],
        deployment: &AcrossDeployment,
        chain: Chain,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain_id: u32 = chain.network_id().parse().unwrap();

        let recipient = if message.is_empty() {
            *wallet_address
        } else {
            Address::from_str(deployment.multicall_handler().as_str()).unwrap()
        };

        let v3_relay_data = V3RelayData {
            depositor: *wallet_address,
            recipient,
            exclusiveRelayer: Address::ZERO,
            inputToken: Address::from_str(input_asset.token_id.clone().unwrap().as_ref()).unwrap(),
            outputToken: *output_token,
            inputAmount: *amount,
            outputAmount: U256::from(100), // safe amount
            originChainId: U256::from(chain_id),
            depositId: u32::MAX,
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(message.to_vec()),
        };
        let value = if is_native { format!("{amount:#x}") } else { String::from("0x0") };
        let data = V3SpokePoolInterface::fillV3RelayCall {
            relayData: v3_relay_data.clone(),
            repaymentChainId: U256::from(chain_id),
        }
        .abi_encode();
        let tx = TransactionObject::new_call_to_value(deployment.spoke_pool, &value, data);
        let gas_limit = self.estimate_gas_transaction(chain, tx).await.unwrap_or(U256::from(Self::get_default_fill_limit(chain)));
        Ok((gas_limit, v3_relay_data))
    }

    fn get_default_fill_limit(chain: Chain) -> u64 {
        match chain {
            Chain::Monad => DEFAULT_FILL_GAS_LIMIT * 3,
            _ => DEFAULT_FILL_GAS_LIMIT,
        }
    }

    async fn usd_price_for_chain(&self, chain: Chain, existing_results: &[IMulticall3::Result]) -> Result<BigInt, SwapperError> {
        let feed = ChainlinkPriceFeed::new_usd_feed_for_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        if chain == Chain::Monad {
            let results = create_eth_client(self.rpc_provider.clone(), Chain::Monad)?
                .multicall3(vec![feed.latest_round_call3()])
                .await
                .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;
            ChainlinkPriceFeed::decoded_answer(&results[0])
        } else {
            ChainlinkPriceFeed::decoded_answer(&existing_results[3])
        }
    }

    pub fn update_v3_relay_data(
        &self,
        v3_relay_data: &mut V3RelayData,
        user_address: &Address,
        output_amount: &U256,
        original_output_asset: &AssetId,
        output_token: &Address,
        timestamp: u32,
        referral_fee: &ReferralFee,
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

    pub fn get_eta_in_seconds(&self, from_chain: &Chain, to_chain: &Chain) -> Option<u32> {
        let from_chain = EVMChain::from_chain(*from_chain)?;
        let to_chain = EVMChain::from_chain(*to_chain)?;
        let from_chain_l2 = from_chain.is_ethereum_layer2();
        let to_chain_l2 = to_chain.is_ethereum_layer2();
        Some(match (from_chain_l2, to_chain_l2) {
            (true, true) => 5,   // L2 to L2
            (true, false) => 10, // L2 to L1
            (false, _) => 20,    // L1 to L2
        })
    }
}

#[async_trait]
impl Swapper for Across {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_WETH.id.clone(), ARBITRUM_USDC.id.clone(), ARBITRUM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_WETH.id.clone(), ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Base, vec![BASE_WETH.id.clone(), BASE_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::Blast, vec![BLAST_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Linea, vec![LINEA_WETH.id.clone(), LINEA_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_WETH.id.clone(), OPTIMISM_USDC.id.clone(), OPTIMISM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Polygon, vec![POLYGON_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_WETH.id.clone(), ZKSYNC_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::World, vec![WORLD_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Ink, vec![INK_WETH.id.clone(), INK_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Unichain, vec![UNICHAIN_WETH.id.clone(), UNICHAIN_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::Monad, vec![MONAD_USDC.id.clone(), MONAD_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_ETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_USDC.id.clone(), HYPEREVM_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Plasma, vec![PLASMA_USDT.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if request.from_asset.chain() == request.to_asset.chain() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_is_native = request.from_asset.is_native();
        let from_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(SwapperError::from)?;
        let wallet_address = eth_address::parse_str(&request.wallet_address)?;

        let _ = AcrossDeployment::deployment_by_chain(&request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_asset = eth_address::convert_native_to_weth(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let output_asset = eth_address::convert_native_to_weth(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let original_output_asset = request.to_asset.asset_id();
        let output_token = eth_address::parse_asset_id(&output_asset)?;

        // Get L1 token address
        let mappings = AcrossDeployment::asset_mappings();
        let asset_mapping = mappings.iter().find(|x| x.set.contains(&input_asset)).unwrap();
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).unwrap();
        let mainnet_token = eth_address::parse_or_weth_address(asset_mainnet, from_chain)?;

        let hubpool_client = HubPoolClient::new(self.rpc_provider.clone(), Chain::Ethereum);
        let config_client = ConfigStoreClient::new(self.rpc_provider.clone(), Chain::Ethereum);

        let calls = vec![
            hubpool_client.paused_call3(),
            hubpool_client.sync_call3(&mainnet_token),
            hubpool_client.pooled_token_call3(&mainnet_token),
        ];
        let results = self.multicall3(hubpool_client.chain, calls).await?;

        // Check if protocol is paused
        let is_paused = hubpool_client.decoded_paused_call3(&results[0])?;
        if is_paused {
            return Err(SwapperError::ComputeQuoteError("Across protocol is paused".into()));
        }

        // Check bridge amount is too large (Across API has some limit in USD amount but we don't have that info)
        if from_amount > hubpool_client.decoded_pooled_token_call3(&results[2])?.liquidReserves {
            return Err(SwapperError::ComputeQuoteError("Bridge amount is too large".into()));
        }

        // Prepare data for lp fee calculation (token config, utilization, current time)
        let token_config_req = config_client.fetch_config(&mainnet_token); // cache is used inside config_client
        let mut calls = vec![
            hubpool_client.utilization_call3(&mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&mainnet_token, from_amount),
            hubpool_client.get_current_time(),
        ];

        let gas_price_feed = ChainlinkPriceFeed::new_usd_feed_for_chain(request.to_asset.chain()).unwrap_or_else(ChainlinkPriceFeed::new_eth_usd_feed);
        if !input_is_native {
            calls.push(gas_price_feed.latest_round_call3());
        }

        let multicall_results = self.multicall3(hubpool_client.chain, calls).await?;
        let token_config = token_config_req.await?;

        let util_before = hubpool_client.decoded_utilization_call3(&multicall_results[0])?;
        let util_after = hubpool_client.decoded_utilization_call3(&multicall_results[1])?;
        let timestamp = hubpool_client.decoded_current_time(&multicall_results[2])?;

        let rate_model = Self::get_rate_model(&input_asset, &output_asset, &token_config);
        let cost_config = &asset_mapping.capital_cost;

        // Calculate lp fee
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent, cost_config.decimals);

        // Calculate relayer fee
        let relayer_calc = RelayerFeeCalculator::default();
        let relayer_fee_percent = relayer_calc.capital_fee_percent(&BigInt::from_str(&request.value).unwrap(), cost_config);
        let relayer_fee = fees::multiply(from_amount, relayer_fee_percent, cost_config.decimals);

        let referral_config = request.options.fee.clone().unwrap_or_default().evm_bridge;

        // Calculate gas limit / price for relayer
        let remain_amount = from_amount - lpfee - relayer_fee;
        let (message, referral_fee) = self.message_for_multicall_handler(&remain_amount, &original_output_asset, &wallet_address, &output_token, &referral_config);

        let gas_price = self.gas_price(request.to_asset.chain()).await?;
        let (gas_limit, mut v3_relay_data) = self
            .estimate_gas_limit(
                &from_amount,
                input_is_native,
                &input_asset,
                &output_token,
                &wallet_address,
                &message,
                &destination_deployment,
                request.to_asset.chain(),
            )
            .await?;
        let mut gas_fee = gas_limit * gas_price;
        if !input_is_native {
            let price = self.usd_price_for_chain(request.to_asset.chain(), &multicall_results).await?;
            gas_fee = Self::calculate_fee_in_token(&gas_fee, &price, 6);
        }

        // Check if bridge amount is too small
        if remain_amount < gas_fee {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }

        let output_amount = remain_amount - gas_fee;
        let to_value = output_amount - referral_fee;

        // Update v3 relay data (was used to estimate gas limit) with final output amount, quote timestamp and referral fee.
        self.update_v3_relay_data(
            &mut v3_relay_data,
            &wallet_address,
            &output_amount,
            &original_output_asset,
            &output_token,
            timestamp,
            &referral_config,
        )?;
        let route_data = HexEncode(v3_relay_data.abi_encode());

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: input_asset.clone(),
                    output: output_asset.clone(),
                    route_data,
                    gas_limit: Some(DEFAULT_DEPOSIT_GAS_LIMIT.to_string()),
                }],
            },
            request: request.clone(),
            eta_in_seconds: self.get_eta_in_seconds(&request.from_asset.chain(), &request.to_asset.chain()),
        })
    }

    async fn fetch_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_chain = quote.request.from_asset.chain();
        let deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let dst_chain_id: u32 = quote.request.to_asset.chain().network_id().parse().unwrap();
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;

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

        let approval: Option<ApprovalData> = {
            if input_is_native {
                None
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    v3_relay_data.inputToken.to_string(),
                    deployment.spoke_pool.into(),
                    v3_relay_data.inputAmount,
                    self.rpc_provider.clone(),
                    &from_chain,
                )
                .await?
                .approval_data()
            }
        };

        let to: String = deployment.spoke_pool.into();
        let mut gas_limit = if approval.is_some() { route.gas_limit.clone() } else { None };

        if matches!(data, FetchQuoteData::EstimateGas) {
            let hex_value = format!("{:#x}", U256::from_str(value).unwrap());
            let tx = TransactionObject::new_call_to_value(&to, &hex_value, deposit_v3_call.clone());
            gas_limit = Some(self.estimate_gas_transaction(from_chain, tx).await?.to_string());
        }

        Ok(SwapperQuoteData::new_contract(
            deployment.spoke_pool.into(),
            value.to_string(),
            HexEncode(deposit_v3_call.clone()),
            approval,
            gas_limit,
        ))
    }
    async fn get_vault_addresses(&self) -> Result<Vec<String>, SwapperError> {
        Ok(AcrossDeployment::vault_addresses())
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let receipt = create_eth_client(self.rpc_provider.clone(), chain)?
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(SwapperError::from)?;

        let origin_chain_id: u64 = chain.network_id().parse().map_err(|_| SwapperError::NotSupportedChain)?;
        let deposit = parse_deposit_from_logs(&receipt.logs, origin_chain_id)?;

        if let Some(destination_chain_id) = deposit.destination_chain_id {
            let destination_chain = Chain::from_chain_id(destination_chain_id).ok_or(SwapperError::NotSupportedChain)?;
            let fill_tx = self.check_fill_on_chain(deposit.origin_chain_id, deposit.deposit_id, destination_chain).await?;

            if fill_tx.is_some() {
                let metadata = Self::build_swap_metadata(&deposit, destination_chain_id);
                Ok(SwapResult {
                    status: primitives::swap::SwapStatus::Completed,
                    metadata,
                })
            } else {
                Ok(SwapResult {
                    status: primitives::swap::SwapStatus::Pending,
                    metadata: None,
                })
            }
        } else {
            let metadata = Self::build_swap_metadata(&deposit, origin_chain_id);
            Ok(SwapResult {
                status: primitives::swap::SwapStatus::Completed,
                metadata,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::SolEvent;
    use gem_evm::{multicall3::IMulticall3, rpc::model::Log};
    use primitives::asset_constants::*;

    #[test]
    fn test_is_supported_pair() {
        let weth_eth = AssetId::from_token(Chain::Ethereum, WETH_ETH_CONTRACT);
        let weth_op = AssetId::from_token(Chain::Optimism, WETH_OP_CONTRACT);
        let weth_arb = AssetId::from_token(Chain::Arbitrum, WETH_ARB_CONTRACT);
        let weth_bsc: AssetId = ETH_SMARTCHAIN_ASSET_ID.into();

        let usdc_eth: AssetId = USDC_ETH_ASSET_ID.into();
        let usdc_arb: AssetId = USDC_ARB_ASSET_ID.into();
        let usdc_monad: AssetId = USDC_MONAD_ASSET_ID.into();
        let usdt_eth: AssetId = USDT_ETH_ASSET_ID.into();
        let usdt_monad: AssetId = USDT_MONAD_ASSET_ID.into();

        assert!(Across::is_supported_pair(&weth_eth, &weth_op));
        assert!(Across::is_supported_pair(&weth_op, &weth_arb));
        assert!(Across::is_supported_pair(&usdc_eth, &usdc_arb));
        assert!(Across::is_supported_pair(&usdc_monad, &usdc_eth));
        assert!(Across::is_supported_pair(&usdt_monad, &usdt_eth));
        assert!(Across::is_supported_pair(&weth_eth, &weth_bsc));

        assert!(!Across::is_supported_pair(&weth_eth, &usdc_eth));

        // native asset
        let eth = AssetId::from(Chain::Ethereum, None);
        let op = AssetId::from(Chain::Optimism, None);
        let arb = AssetId::from(Chain::Arbitrum, None);
        let linea = AssetId::from(Chain::Linea, None);

        assert!(Across::is_supported_pair(&eth, &linea));
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

    #[test]
    fn test_resolve_token_asset_native_eth_via_weth() {
        let result = resolve_token_asset(Chain::Ethereum, WETH_ETH_CONTRACT);
        assert_eq!(result, Some(AssetId::from_chain(Chain::Ethereum)));
    }

    #[test]
    fn test_resolve_token_asset_native_arb_via_weth() {
        let result = resolve_token_asset(Chain::Arbitrum, WETH_ARB_CONTRACT);
        assert_eq!(result, Some(AssetId::from_chain(Chain::Arbitrum)));
    }

    #[test]
    fn test_resolve_token_asset_usdc_checksummed() {
        let result = resolve_token_asset(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        assert_eq!(result, Some(AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")));
    }

    #[test]
    fn test_resolve_token_asset_unsupported_chain() {
        let result = resolve_token_asset(Chain::Bitcoin, "0x123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_v3_funds_deposited() {
        let input_amount = U256::from(1_000_000_000_000_000_000u64);
        let output_amount = U256::from(999_000_000_000_000_000u64);
        let log = build_event_log(
            V3SpokePoolInterface::V3FundsDeposited::SIGNATURE_HASH,
            &[42161, 12345, 0],
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
            input_amount,
            output_amount,
        );

        let result = parse_deposit_from_logs(&[log], 1).unwrap();
        assert_eq!(result.deposit_id, 12345);
        assert_eq!(result.origin_chain_id, 1);
        assert_eq!(result.destination_chain_id.unwrap(), 42161);
        assert_eq!(result.input_token.unwrap(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        assert_eq!(result.output_token.unwrap(), "0x82af49447d8a07e3bd95bd0d56f35241523fbab1");
        assert_eq!(result.input_amount.unwrap(), input_amount.to_string());
        assert_eq!(result.output_amount.unwrap(), output_amount.to_string());
    }

    #[test]
    fn test_parse_new_funds_deposited() {
        let input_amount = U256::from(20_000_000_000_000_000u64);
        let output_amount = U256::from(19_900_000_000_000_000u64);
        let log = build_event_log(
            V3SpokePoolInterface::FundsDeposited::SIGNATURE_HASH,
            &[8453, 5452553, 0],
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0x4200000000000000000000000000000000000006",
            input_amount,
            output_amount,
        );

        let result = parse_deposit_from_logs(&[log], 1).unwrap();
        assert_eq!(result.deposit_id, 5452553);
        assert_eq!(result.destination_chain_id.unwrap(), 8453);
        assert_eq!(result.input_token.unwrap(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        assert_eq!(result.output_token.unwrap(), "0x4200000000000000000000000000000000000006");
        assert_eq!(result.input_amount.unwrap(), input_amount.to_string());
        assert_eq!(result.output_amount.unwrap(), output_amount.to_string());
    }

    #[test]
    fn test_parse_filled_relay() {
        let input_amount = U256::from(20_200_000_000_000_000u64);
        let output_amount = U256::from(20_197_000_000_000_000u64);
        let log = build_event_log(
            V3SpokePoolInterface::FilledRelay::SIGNATURE_HASH,
            &[1, 3708468, 0],
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0x4200000000000000000000000000000000000006",
            input_amount,
            output_amount,
        );

        let result = parse_deposit_from_logs(&[log], 8453).unwrap();
        assert_eq!(result.deposit_id, 3708468);
        assert_eq!(result.origin_chain_id, 1);
        assert!(result.destination_chain_id.is_none());
        assert_eq!(result.input_token.unwrap(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        assert_eq!(result.output_token.unwrap(), "0x4200000000000000000000000000000000000006");
    }

    #[test]
    fn test_parse_no_matching_event() {
        let log = Log {
            address: String::new(),
            topics: vec!["0xdeadbeef".into()],
            data: "0x".into(),
            transaction_hash: None,
        };
        assert!(parse_deposit_from_logs(&[log], 1).is_err());
        assert!(parse_deposit_from_logs(&[], 1).is_err());
    }

    fn build_event_log(signature: alloy_primitives::FixedBytes<32>, indexed: &[u64], input_token: &str, output_token: &str, input_amount: U256, output_amount: U256) -> Log {
        let mut data = Vec::new();
        let mut buf = [0u8; 32];
        let input_bytes = alloy_primitives::hex::decode(input_token.strip_prefix("0x").unwrap_or(input_token)).unwrap();
        buf[32 - input_bytes.len()..].copy_from_slice(&input_bytes);
        data.extend_from_slice(&buf);
        buf = [0u8; 32];
        let output_bytes = alloy_primitives::hex::decode(output_token.strip_prefix("0x").unwrap_or(output_token)).unwrap();
        buf[32 - output_bytes.len()..].copy_from_slice(&output_bytes);
        data.extend_from_slice(&buf);
        data.extend_from_slice(&input_amount.to_be_bytes::<32>());
        data.extend_from_slice(&output_amount.to_be_bytes::<32>());
        data.extend_from_slice(&[0u8; 512]);

        let mut topics = vec![format!("{:#x}", signature)];
        for t in indexed {
            topics.push(format!("{:#066x}", t));
        }

        Log {
            address: String::new(),
            topics,
            data: HexEncode(&data),
            transaction_hash: None,
        }
    }

    #[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
    mod swap_integration_tests {
        use super::*;
        use crate::{
            FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError, SwapperMode,
            config::{ReferralFee, ReferralFees},
        };
        use primitives::{AssetId, Chain, swap::SwapStatus};
        use std::{sync::Arc, time::SystemTime};

        #[tokio::test]
        async fn test_across_quote() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let mut options = Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                })),
                preferred_providers: vec![],
                use_max_amount: false,
            };
            options.fee.as_mut().unwrap().evm_bridge = ReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            };

            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Optimism).into(),
                to_asset: AssetId::from_chain(Chain::Arbitrum).into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: "20000000000000000".into(), // 0.02 ETH
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

        #[tokio::test]
        async fn test_across_quote_eth_usdc_to_monad_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                fee: None,
                preferred_providers: vec![],
                use_max_amount: false,
            };

            let wallet = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let from_asset: AssetId = USDC_ETH_ASSET_ID.into();
            let to_asset: AssetId = USDC_MONAD_ASSET_ID.into();
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: "50000000".into(), // 50 USDC
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

        #[tokio::test]
        async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::new(network_provider.clone());

            let tx_hash = "0x2ed43336441c830e859dada09e6eee6b5ee5b160e0e420fdf17f6e46dc240e88";
            let chain = Chain::Base;

            let result = swap_provider.get_swap_result(chain, tx_hash).await?;

            println!("Across swap result: {:?}", result);
            assert_eq!(result.status, SwapStatus::Completed);

            let metadata = result.metadata.unwrap();
            assert_eq!(metadata.provider, Some("across".to_string()));
            assert!(!metadata.from_value.is_empty());
            assert!(!metadata.to_value.is_empty());

            Ok(())
        }
    }
}
