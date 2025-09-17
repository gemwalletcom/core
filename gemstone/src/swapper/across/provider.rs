use super::{
    api::AcrossApi,
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
    DEFAULT_FILL_TIMEOUT,
};
use crate::{
    config::swap_config::SwapReferralFee,
    debug_println,
    ethereum::jsonrpc as eth_rpc,
    network::AlienProvider,
    swapper::{
        across::{DEFAULT_DEPOSIT_GAS_LIMIT, DEFAULT_FILL_GAS_LIMIT},
        approval::check_approval_erc20,
        asset::*,
        chainlink::ChainlinkPriceFeed,
        eth_address,
        models::*,
        Swapper, SwapperApprovalData, SwapperError, SwapperProvider, SwapperQuoteData, SwapperSwapResult,
    },
};
use alloy_primitives::{
    hex::{decode as HexDecode, encode_prefixed as HexEncode},
    Address, Bytes, FixedBytes, U256,
};
use alloy_sol_types::{SolCall, SolValue};

use crate::network::jsonrpc_client_with_chain;
use async_trait::async_trait;
use bs58;
use gem_evm::{
    across::{
        contracts::{
            multicall_handler,
            V3SpokePoolInterface::{self, V3RelayData},
        },
        deployment::AcrossDeployment,
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    contracts::erc20::IERC20,
    jsonrpc::TransactionObject,
    multicall3::IMulticall3,
    weth::WETH9,
};
use gem_solana::{jsonrpc::SolanaRpc, models::prioritization_fee::SolanaPrioritizationFee};
use num_bigint::{BigInt, Sign};
use primitives::{swap::SwapStatus, AssetId, Chain, ChainType, EVMChain};
use std::{fmt::Debug, str::FromStr, sync::Arc};

const DEFAULT_SOLANA_COMPUTE_LIMIT: u64 = 200_000;

#[derive(Debug)]
pub struct Across {
    pub provider: SwapperProviderType,
}

impl Default for Across {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Across),
        }
    }
}

impl Across {
    pub fn boxed() -> Box<dyn Swapper> {
        Box::new(Self::default())
    }

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        if from_asset.chain == Chain::Solana {
            return false;
        }

        if to_asset.chain == Chain::Solana {
            if to_asset != &SOLANA_USDC.id {
                return false;
            }
            // Check if from_asset is a supported USDC token on EVM chains
            let from_normalized = match eth_address::normalize_weth_asset(from_asset) {
                Some(asset) => asset,
                None => return false,
            };
            return AcrossDeployment::asset_mappings()
                .into_iter()
                .any(|mapping| mapping.set.contains(&from_normalized) && mapping.set.contains(&SOLANA_USDC.id));
        }

        let from = match eth_address::normalize_weth_asset(from_asset) {
            Some(asset) => asset,
            None => return false,
        };
        let to = match eth_address::normalize_weth_asset(to_asset) {
            Some(asset) => asset,
            None => return false,
        };
        AcrossDeployment::asset_mappings()
            .into_iter()
            .any(|x| x.set.contains(&from) && x.set.contains(&to))
    }

    fn decode_address_bytes32(addr: &Address) -> FixedBytes<32> {
        let mut bytes = [0u8; 32];
        bytes[12..32].copy_from_slice(addr.as_slice());
        FixedBytes::from(bytes)
    }

    fn decode_bs58_bytes32(addr: &str) -> Result<FixedBytes<32>, SwapperError> {
        let decoded = bs58::decode(addr).into_vec().map_err(|_| SwapperError::InvalidAddress(addr.to_string()))?;
        if decoded.len() != 32 {
            return Err(SwapperError::InvalidAddress(addr.to_string()));
        }
        let bytes: [u8; 32] = decoded.try_into().map_err(|_| SwapperError::InvalidAddress(addr.to_string()))?;
        Ok(FixedBytes::from(bytes))
    }

    fn token_bytes32_for_asset(asset: &AssetId) -> Result<FixedBytes<32>, SwapperError> {
        match asset.chain.chain_type() {
            ChainType::Solana => {
                let id = asset
                    .token_id
                    .as_deref()
                    .ok_or_else(|| SwapperError::InvalidAddress("missing token_id for Solana".into()))?;
                Self::decode_bs58_bytes32(id)
            }
            ChainType::Ethereum => {
                let evm_chain = EVMChain::from_chain(asset.chain).ok_or(SwapperError::NotSupportedChain)?;
                let default_weth = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;
                let id = if asset.is_native() {
                    default_weth
                } else {
                    asset.token_id.as_deref().unwrap()
                };
                Ok(Self::decode_address_bytes32(&Address::from_str(id).unwrap()))
            }
            _ => Err(SwapperError::NotImplemented),
        }
    }

    fn is_solana_destination(request: &SwapperQuoteRequest) -> bool {
        request.to_asset.chain() == Chain::Solana
    }

    fn get_output_asset(request: &SwapperQuoteRequest) -> Result<AssetId, SwapperError> {
        if Self::is_solana_destination(request) {
            Ok(request.to_asset.asset_id())
        } else {
            let norm_output_asset = eth_address::normalize_weth_asset(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedPair)?;
            Ok(norm_output_asset)
        }
    }

    fn get_destination_chain_id(chain: &Chain) -> Result<u64, SwapperError> {
        let deployment = AcrossDeployment::deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        Ok(deployment.chain_id)
    }

    fn calculate_relayer_fee_for_destination(
        request: &SwapperQuoteRequest,
        from_amount: U256,
        cost_config: &fees::CapitalCostConfig,
        sol_price: Option<&BigInt>,
    ) -> U256 {
        if Self::is_solana_destination(request) {
            if let Some(sol_usd_price) = sol_price {
                // 0.000005 SOL in lamports (9 decimals) = 5000 lamports
                let sol_fee_amount = U256::from(5000_u64);
                Self::calculate_fee_in_token(&sol_fee_amount, sol_usd_price, 6)
            } else {
                // Fallback to hardcoded value if price is not available
                U256::from(5000)
            }
        } else {
            let relayer_calc = RelayerFeeCalculator::default();
            let from_amount_bigint = BigInt::from_bytes_le(Sign::Plus, &from_amount.to_le_bytes::<32>());
            let relayer_fee_percent = relayer_calc.capital_fee_percent(&from_amount_bigint, cost_config);
            fees::multiply(from_amount, relayer_fee_percent, cost_config.decimals)
        }
    }

    pub fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    /// Return (message, referral_fee)
    pub fn message_for_evm_multicall_handler(
        &self,
        amount: &U256,
        original_output_asset: &AssetId,
        output_token: &Address,
        user_address: &Address,
        referral_fee: &SwapReferralFee,
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

    /// Return (message, referral_fee) depending on destination chain type
    pub fn message_for_multicall_handler(
        &self,
        amount: &U256,
        original_output_asset: &AssetId,
        output_token_evm: Option<&Address>,
        user_address: &Address,
        referral_fee: &SwapReferralFee,
    ) -> (Vec<u8>, U256) {
        match original_output_asset.chain.chain_type() {
            ChainType::Ethereum => {
                if let Some(token) = output_token_evm {
                    self.message_for_evm_multicall_handler(amount, original_output_asset, token, user_address, referral_fee)
                } else {
                    (vec![], U256::from(0))
                }
            }
            ChainType::Solana => (vec![], U256::from(0)),
            _ => (vec![], U256::from(0)),
        }
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

    fn erc20_transfer_calls(
        token: &Address,
        user_address: &Address,
        user_amount: &U256,
        fee_address: &Address,
        fee_amount: &U256,
    ) -> Vec<multicall_handler::Call> {
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
        output_token: FixedBytes<32>,
        wallet_address: &Address,
        message: &[u8],
        deployment: &AcrossDeployment,
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain_id = Self::get_destination_chain_id(&chain)?;

        // Prepare bytes32 fields
        let depositor = Self::decode_address_bytes32(wallet_address);
        let recipient_evm = if message.is_empty() {
            *wallet_address
        } else {
            Address::from_str(deployment.multicall_handler().as_str()).unwrap()
        };
        let recipient = Self::decode_address_bytes32(&recipient_evm);
        let input_token = Self::token_bytes32_for_asset(input_asset)?;

        let v3_relay_data = V3RelayData {
            depositor,
            recipient,
            exclusiveRelayer: FixedBytes::from([0u8; 32]),
            inputToken: input_token,
            outputToken: output_token,
            inputAmount: *amount,
            outputAmount: U256::from(100), // safe amount
            originChainId: U256::from(chain_id),
            depositId: U256::from(u32::MAX),
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(message.to_vec()),
        };
        let value = if is_native { format!("{amount:#x}") } else { String::from("0x0") };
        let data = V3SpokePoolInterface::fillRelayCall {
            relayData: v3_relay_data.clone(),
            repaymentChainId: U256::from(chain_id),
            repaymentAddress: Self::decode_address_bytes32(wallet_address),
        }
        .abi_encode();
        if chain.chain_type() != ChainType::Ethereum {
            return Err(SwapperError::NotImplemented);
        }
        let tx = TransactionObject::new_call_to_value(deployment.spoke_pool, &value, data);
        let gas_limit = eth_rpc::estimate_gas(provider, chain, tx).await;
        Ok((gas_limit.unwrap_or(U256::from(DEFAULT_FILL_GAS_LIMIT)), v3_relay_data))
    }

    pub fn update_v3_relay_data(
        &self,
        v3_relay_data: &mut V3RelayData,
        user_address: &Address,
        output_amount: &U256,
        original_output_asset: &AssetId,
        output_token: Option<&Address>,
        timestamp: u32,
        referral_fee: &SwapReferralFee,
    ) -> Result<U256, SwapperError> {
        let (message, final_referral_fee) = self.message_for_multicall_handler(output_amount, original_output_asset, output_token, user_address, referral_fee);

        v3_relay_data.outputAmount = *output_amount;
        v3_relay_data.fillDeadline = timestamp + DEFAULT_FILL_TIMEOUT;
        v3_relay_data.message = message.into();

        Ok(final_referral_fee)
    }

    pub fn calculate_fee_in_token(fee_in_wei: &U256, token_price: &BigInt, token_decimals: u32) -> U256 {
        let fee = BigInt::from_bytes_le(Sign::Plus, &fee_in_wei.to_le_bytes::<32>());
        let fee_in_token = fee * token_price * BigInt::from(10_u64.pow(token_decimals)) / BigInt::from(10_u64.pow(8)) / BigInt::from(10_u64.pow(18));
        U256::from_le_slice(&fee_in_token.to_bytes_le().1)
    }

    async fn fetch_solana_unit_price(provider: Arc<dyn AlienProvider>) -> Result<u64, SwapperError> {
        let client = jsonrpc_client_with_chain(provider, Chain::Solana);
        let rpc_call = SolanaRpc::GetRecentPrioritizationFees;
        let fees: Vec<SolanaPrioritizationFee> = client.request(rpc_call).await?;

        if fees.is_empty() {
            return Err(SwapperError::NetworkError("Failed to fetch recent prioritization fees".to_string()));
        }

        // Calculate average prioritization fee from recent transactions
        let total_fee: u64 = fees.iter().map(|f| f.prioritization_fee as u64).sum();
        let average_fee = total_fee / fees.len() as u64;

        // Return at least 1 microlamport per compute unit
        Ok(std::cmp::max(1, average_fee))
    }

    async fn calculate_gas_price_and_fee(
        &self,
        gas_chain: Chain,
        from_amount: &U256,
        input_is_native: bool,
        input_asset: &AssetId,
        output_token: FixedBytes<32>,
        wallet_address: &Address,
        message: &[u8],
        destination_deployment: &AcrossDeployment,
        provider: Arc<dyn AlienProvider>,
        eth_price_index: Option<usize>,
        multicall_results: &[IMulticall3::Result],
    ) -> Result<(U256, V3RelayData), SwapperError> {
        if gas_chain == Chain::Solana {
            let unit_price = Self::fetch_solana_unit_price(provider.clone()).await?;
            let gas_fee = DEFAULT_SOLANA_COMPUTE_LIMIT * unit_price;

            let chain_id = Self::get_destination_chain_id(&gas_chain)?;
            let recipient_evm = if message.is_empty() {
                *wallet_address
            } else {
                Address::from_str(destination_deployment.multicall_handler().as_str()).unwrap()
            };
            let v3_relay_data = V3RelayData {
                depositor: Self::decode_address_bytes32(wallet_address),
                recipient: Self::decode_address_bytes32(&recipient_evm),
                exclusiveRelayer: FixedBytes::from([0u8; 32]),
                inputToken: Self::token_bytes32_for_asset(input_asset)?,
                outputToken: output_token,
                inputAmount: *from_amount,
                outputAmount: U256::from(100), // safe amount
                originChainId: U256::from(chain_id),
                depositId: U256::from(u32::MAX),
                fillDeadline: u32::MAX,
                exclusivityDeadline: 0,
                message: Bytes::from(message.to_vec()),
            };

            Ok((U256::from(gas_fee), v3_relay_data))
        } else {
            let gas_price_req = eth_rpc::fetch_gas_price(provider.clone(), gas_chain);
            let gas_limit_req = self.estimate_gas_limit(
                from_amount,
                input_is_native,
                input_asset,
                output_token,
                wallet_address,
                message,
                destination_deployment,
                provider.clone(),
                gas_chain,
            );

            let (tuple, gas_price) = futures::join!(gas_limit_req, gas_price_req);
            let (gas_limit, v3_relay_data) = tuple?;
            let mut gas_fee = gas_limit * gas_price?;

            if let Some(index) = eth_price_index {
                let eth_price = ChainlinkPriceFeed::decoded_answer(&multicall_results[index])?;
                gas_fee = Self::calculate_fee_in_token(&gas_fee, &eth_price, 6);
            }

            Ok((gas_fee, v3_relay_data))
        }
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
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(
                Chain::Arbitrum,
                vec![ARBITRUM_WETH.id.clone(), ARBITRUM_USDC.id.clone(), ARBITRUM_USDT.id.clone()],
            ),
            SwapperChainAsset::Assets(
                Chain::Ethereum,
                vec![ETHEREUM_WETH.id.clone(), ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone()],
            ),
            SwapperChainAsset::Assets(Chain::Base, vec![BASE_WETH.id.clone(), BASE_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::Blast, vec![BLAST_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Linea, vec![LINEA_WETH.id.clone(), LINEA_USDT.id.clone()]),
            SwapperChainAsset::Assets(
                Chain::Optimism,
                vec![OPTIMISM_WETH.id.clone(), OPTIMISM_USDC.id.clone(), OPTIMISM_USDT.id.clone()],
            ),
            SwapperChainAsset::Assets(Chain::Polygon, vec![POLYGON_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_WETH.id.clone(), ZKSYNC_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::World, vec![WORLD_WETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Ink, vec![INK_WETH.id.clone(), INK_USDT.id.clone()]),
            SwapperChainAsset::Assets(Chain::Unichain, vec![UNICHAIN_WETH.id.clone(), UNICHAIN_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_ETH.id.clone()]),
            SwapperChainAsset::Assets(Chain::Solana, vec![SOLANA_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError> {
        if request.from_asset.chain() == request.to_asset.chain() {
            return Err(SwapperError::NotSupportedPair);
        }

        if request.from_asset.chain() == Chain::Solana {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_is_native = request.from_asset.is_native();
        let from_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(SwapperError::from)?;
        let wallet_address = eth_address::parse_str(&request.wallet_address)?;

        let _ = AcrossDeployment::deployment_by_chain(&request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_asset = eth_address::normalize_weth_asset(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedPair)?;
        let output_asset = Self::get_output_asset(request)?;
        let original_output_asset = request.to_asset.asset_id();

        // Get L1 token address
        let mappings = AcrossDeployment::asset_mappings();
        let asset_mapping = mappings.iter().find(|x| x.set.contains(&input_asset)).unwrap();
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).unwrap();
        let mainnet_token = eth_address::normalize_weth_address(asset_mainnet, from_chain)?;

        let hubpool_client = HubPoolClient::new(provider.clone(), Chain::Ethereum);
        let config_client = ConfigStoreClient::new(provider.clone(), Chain::Ethereum);

        let calls = vec![
            hubpool_client.paused_call3(),
            hubpool_client.sync_call3(&mainnet_token),
            hubpool_client.pooled_token_call3(&mainnet_token),
        ];
        let results = eth_rpc::multicall3_call(provider.clone(), &hubpool_client.chain, calls).await?;

        let is_paused = hubpool_client.decoded_paused_call3(&results[0])?;
        if is_paused {
            return Err(SwapperError::ComputeQuoteError("Across protocol is paused".into()));
        }

        if from_amount > hubpool_client.decoded_pooled_token_call3(&results[2])?.liquidReserves {
            return Err(SwapperError::ComputeQuoteError("Bridge amount is too large".into()));
        }

        let token_config_req = config_client.fetch_config(&mainnet_token); // cache is used inside config_client
        let mut calls = vec![
            hubpool_client.utilization_call3(&mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&mainnet_token, from_amount),
            hubpool_client.get_current_time(),
        ];

        let eth_price_feed = ChainlinkPriceFeed::new_eth_usd_feed(provider.clone());
        let sol_price_feed = ChainlinkPriceFeed::new_sol_usd_feed(provider.clone());
        let mut next_call_index = 3; // utilization(0), utilization(from_amount), current_time
        let eth_price_index = if !input_is_native {
            calls.push(eth_price_feed.latest_round_call3());
            let index = next_call_index;
            next_call_index += 1;
            Some(index)
        } else {
            None
        };
        let sol_price_index = if Self::is_solana_destination(request) {
            calls.push(sol_price_feed.latest_round_call3());
            let index = next_call_index;
            Some(index)
        } else {
            None
        };

        let multicall_req = eth_rpc::multicall3_call(provider.clone(), &hubpool_client.chain, calls);

        let batch_results = futures::join!(token_config_req, multicall_req);
        let token_config = batch_results.0?;
        let multicall_results = batch_results.1?;

        let util_before = hubpool_client.decoded_utilization_call3(&multicall_results[0])?;
        let util_after = hubpool_client.decoded_utilization_call3(&multicall_results[1])?;
        let timestamp = hubpool_client.decoded_current_time(&multicall_results[2])?;

        let rate_model = Self::get_rate_model(&input_asset, &output_asset, &token_config);
        let cost_config = &asset_mapping.capital_cost;

        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent, cost_config.decimals);
        debug_println!("lpfee: {}", lpfee);

        let sol_price = if let Some(index) = sol_price_index {
            Some(ChainlinkPriceFeed::decoded_answer(&multicall_results[index])?)
        } else {
            None
        };

        let relayer_fee = Self::calculate_relayer_fee_for_destination(request, from_amount, cost_config, sol_price.as_ref());
        debug_println!("relayer_fee: {}", relayer_fee);

        let referral_config = request.options.fee.clone().unwrap_or_default().evm_bridge;

        let remain_amount = from_amount - lpfee - relayer_fee;
        let output_token_evm = if request.to_asset.chain().chain_type() == ChainType::Ethereum {
            Some(eth_address::parse_asset_id(&output_asset)?)
        } else {
            None
        };
        let (message, _estimation_referral_fee) = self.message_for_multicall_handler(
            &remain_amount,
            &original_output_asset,
            output_token_evm.as_ref(),
            &wallet_address,
            &referral_config,
        );

        let gas_chain = request.to_asset.chain();
        let output_token_bytes = Self::token_bytes32_for_asset(&output_asset)?;
        let (gas_fee, mut v3_relay_data) = self
            .calculate_gas_price_and_fee(
                gas_chain,
                &from_amount,
                input_is_native,
                &input_asset,
                output_token_bytes,
                &wallet_address,
                &message,
                &destination_deployment,
                provider.clone(),
                eth_price_index,
                &multicall_results,
            )
            .await?;
        debug_println!("gas_fee: {}", gas_fee);

        if remain_amount < gas_fee {
            return Err(SwapperError::InputAmountTooSmall);
        }

        let output_amount = remain_amount - gas_fee;
        let final_referral_fee = self.update_v3_relay_data(
            &mut v3_relay_data,
            &wallet_address,
            &output_amount,
            &original_output_asset,
            output_token_evm.as_ref(),
            timestamp,
            &referral_config,
        )?;
        let to_value = output_amount - final_referral_fee;

        let encoded_data = v3_relay_data.abi_encode();
        let route_data = HexEncode(encoded_data);

        Ok(SwapperQuote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: SwapperProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![SwapperRoute {
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

    async fn fetch_quote_data(&self, quote: &SwapperQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_chain = quote.request.from_asset.chain();
        let deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let dst_chain_id = Self::get_destination_chain_id(&quote.request.to_asset.chain())?;
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let depositor = Self::decode_address_bytes32(&eth_address::parse_str(&quote.request.wallet_address)?);
        let recipient = if quote.request.to_asset.chain() == Chain::Solana {
            Self::decode_bs58_bytes32(&quote.request.destination_address)?
        } else {
            let recipient_evm = if v3_relay_data.message.is_empty() {
                eth_address::parse_str(&quote.request.wallet_address)?
            } else {
                Address::from_str(deployment.multicall_handler().as_str()).unwrap()
            };
            Self::decode_address_bytes32(&recipient_evm)
        };

        // input token uses bytes32 (EVM padded or Solana raw depending on origin chain)
        let input_asset_id = quote.request.from_asset.asset_id();
        let input_token = Self::token_bytes32_for_asset(&input_asset_id)?;

        // output token may be EVM or Solana depending on destination chain
        let to_asset_id = quote.request.to_asset.asset_id();
        let output_token = Self::token_bytes32_for_asset(&to_asset_id)?;

        let deposit_v3_call = V3SpokePoolInterface::depositCall {
            depositor,
            recipient,
            inputToken: input_token,
            outputToken: output_token,
            inputAmount: v3_relay_data.inputAmount,
            outputAmount: v3_relay_data.outputAmount,
            destinationChainId: U256::from(dst_chain_id),
            exclusiveRelayer: FixedBytes::from([0u8; 32]),
            quoteTimestamp: v3_relay_data.fillDeadline - DEFAULT_FILL_TIMEOUT,
            fillDeadline: v3_relay_data.fillDeadline,
            exclusivityDeadline: 0,
            message: v3_relay_data.message,
        }
        .abi_encode();

        let input_is_native = quote.request.from_asset.is_native();
        let value: &str = if input_is_native { &quote.from_value } else { "0" };

        let approval: Option<SwapperApprovalData> = {
            if input_is_native {
                None
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    eth_address::parse_asset_id(&quote.request.from_asset.asset_id())?.to_string(),
                    deployment.spoke_pool.into(),
                    v3_relay_data.inputAmount,
                    provider.clone(),
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
            let _gas_limit = eth_rpc::estimate_gas(provider, from_chain, tx).await?;
            debug_println!("gas_limit: {:?}", _gas_limit);
            gas_limit = Some(_gas_limit.to_string());
        }

        let quote_data = SwapperQuoteData {
            to: deployment.spoke_pool.into(),
            value: value.to_string(),
            data: HexEncode(deposit_v3_call.clone()),
            approval,
            gas_limit,
        };

        Ok(quote_data)
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<SwapperSwapResult, SwapperError> {
        let api = AcrossApi::new(provider.clone());
        let status = api.deposit_status(chain, transaction_hash).await?;

        let swap_status = status.swap_status();
        let destination_chain = Chain::from_chain_id(status.destination_chain_id);

        // Determine the transaction hash to show based on status
        let (to_chain, to_tx_hash) = match swap_status {
            SwapStatus::Completed => (destination_chain, status.fill_tx.clone()),
            SwapStatus::Failed | SwapStatus::Refunded => (Some(chain), None),
            SwapStatus::Pending => (destination_chain, None),
        };

        Ok(SwapperSwapResult {
            status: swap_status,
            from_chain: chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain,
            to_tx_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::mock::AlienProviderMock;
    use gem_evm::{
        across::contracts::{multicall_handler, V3SpokePoolInterface::depositCall},
        contracts::erc20::IERC20,
        multicall3::IMulticall3,
        weth::WETH9,
    };
    use primitives::asset_constants::*;
    use std::time::Duration;

    #[test]
    fn test_is_supported_pair() {
        let weth_eth: AssetId = WETH_ETH_ASSET_ID.into();
        let weth_op: AssetId = WETH_OP_ASSET_ID.into();
        let weth_arb: AssetId = WETH_ARB_ASSET_ID.into();
        let weth_bsc: AssetId = ETH_SMARTCHAIN_ASSET_ID.into();

        let usdc_eth: AssetId = USDC_ETH_ASSET_ID.into();
        let usdc_arb: AssetId = USDC_ARB_ASSET_ID.into();

        // EVM -> EVM pairs
        assert!(Across::is_supported_pair(&weth_eth, &weth_op));
        assert!(Across::is_supported_pair(&weth_op, &weth_arb));
        assert!(Across::is_supported_pair(&usdc_eth, &usdc_arb));
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

        // EVM -> Solana pairs
        let solana_usdc = SOLANA_USDC.id.clone();

        assert!(Across::is_supported_pair(&usdc_eth, &solana_usdc));
        assert!(Across::is_supported_pair(&usdc_arb, &solana_usdc));

        let solana_usdt = AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
        assert!(!Across::is_supported_pair(&usdc_eth, &solana_usdt)); // Only USDC supported

        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_eth));
        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_arb));
        assert!(!Across::is_supported_pair(&weth_eth, &solana_usdc));
    }

    #[test]
    fn test_solana_address_to_bytes32() {
        let bytes = Across::decode_bs58_bytes32("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let expected = "0xc6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61";

        assert_eq!(HexEncode(bytes), expected);

        let bytes = Across::decode_bs58_bytes32("G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR").unwrap();
        let expected = "0xe074190d46821cf0b318d4503f63178e25d76cc7d9d2498d54781fb95bb68868";

        assert_eq!(HexEncode(bytes), expected);
    }

    #[test]
    fn test_v3_relay_data_solana_encoding() {
        // https://etherscan.io/tx/0xd2f84832c9e05ed6b9c1685e253c50c77d52334e354c8af665c7d1159946919b
        let depositor_addr = Address::from_str("0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7").unwrap();
        let input_token_addr = Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap(); // USDC on Arbitrum
        let depositor = Across::decode_address_bytes32(&depositor_addr);
        let recipient = Across::decode_bs58_bytes32("G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR").unwrap();
        let input_token = Across::decode_address_bytes32(&input_token_addr);
        let output_token = Across::decode_bs58_bytes32("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let call = depositCall {
            depositor,
            recipient,
            inputToken: input_token,
            outputToken: output_token,
            inputAmount: U256::from(7000000_u64),
            outputAmount: U256::from(6997408_u64),
            destinationChainId: U256::from(34268394551451_u64),
            exclusiveRelayer: FixedBytes::from([0u8; 32]),
            quoteTimestamp: 1756299179,
            fillDeadline: 1756311051,
            exclusivityDeadline: 0,
            message: Bytes::new(),
        };
        let encoded_call = call.abi_encode();
        let call_data = "0xad5425c6000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7e074190d46821cf0b318d4503f63178e25d76cc7d9d2498d54781fb95bb68868000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d6100000000000000000000000000000000000000000000000000000000006acfc000000000000000000000000000000000000000000000000000000000006ac5a000000000000000000000000000000000000000000000000000001f2abb7bf89b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000068aeffab0000000000000000000000000000000000000000000000000000000068af2e0b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000000";

        assert_eq!(HexEncode(encoded_call), call_data);
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
    fn test_message_for_multicall_handler_eth_to_base() {
        // ETH from Ethereum to Base (destination native -> unwrap WETH)
        let across = Across::default();
        let amount = U256::from_str("1000000000000000000").unwrap(); // 1 ETH

        // Destination is Base native ETH
        let original_output_asset = AssetId::from(Chain::Base, None);
        let output_token_evm = Address::from_str("0x4200000000000000000000000000000000000006").unwrap(); // Base WETH
        let user_address = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();
        let fee_address_str = "0x2222222222222222222222222222222222222222";
        let referral_fee = SwapReferralFee {
            address: fee_address_str.into(),
            bps: 100, // 1%
        };

        let (message, fee_amount) =
            across.message_for_multicall_handler(&amount, &original_output_asset, Some(&output_token_evm), &user_address, &referral_fee);

        // Validate fee math
        let expected_fee = amount * U256::from(100u64) / U256::from(10000u64);
        let expected_user_amount = amount - expected_fee;

        assert_eq!(fee_amount, expected_fee);

        // Decode and validate instructions
        let instructions = multicall_handler::Instructions::abi_decode(&message).unwrap();

        assert_eq!(instructions.fallbackRecipient, user_address);
        assert_eq!(instructions.calls.len(), 3);

        // Call[0]: WETH.withdraw(amount)
        let expected_withdraw = WETH9::withdrawCall { wad: amount }.abi_encode();

        assert_eq!(instructions.calls[0].target, output_token_evm);
        assert_eq!(instructions.calls[0].callData, Bytes::from(expected_withdraw));
        assert_eq!(instructions.calls[0].value, U256::from(0));

        // Call[1]: send ETH to user
        assert_eq!(instructions.calls[1].target, user_address);
        assert!(instructions.calls[1].callData.is_empty());
        assert_eq!(instructions.calls[1].value, expected_user_amount);

        // Call[2]: send ETH to referral address
        let fee_address = Address::from_str(fee_address_str).unwrap();

        assert_eq!(instructions.calls[2].target, fee_address);
        assert!(instructions.calls[2].callData.is_empty());
        assert_eq!(instructions.calls[2].value, expected_fee);
    }

    #[test]
    fn test_message_for_multicall_handler_usdc_arb_to_op() {
        // USDC from Arbitrum to Optimism (destination ERC20 -> transfer calls)
        let across = Across::default();
        let amount = U256::from(1_000_000u64); // 1 USDC (6 decimals)

        // Destination is Optimism USDC (ERC20)
        let original_output_asset: AssetId = USDC_OP_ASSET_ID.into();
        let output_token_evm = Address::from_str("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85").unwrap(); // OP USDC
        let user_address = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();
        let fee_address_str = "0x2222222222222222222222222222222222222222";
        let referral_fee = SwapReferralFee {
            address: fee_address_str.into(),
            bps: 100, // 1%
        };

        let (message, fee_amount) =
            across.message_for_multicall_handler(&amount, &original_output_asset, Some(&output_token_evm), &user_address, &referral_fee);

        // Validate fee math
        let expected_fee = amount * U256::from(100u64) / U256::from(10000u64);
        let expected_user_amount = amount - expected_fee;

        assert_eq!(fee_amount, expected_fee);

        // Decode and validate instructions
        let instructions = multicall_handler::Instructions::abi_decode(&message).unwrap();

        assert_eq!(instructions.fallbackRecipient, user_address);
        assert_eq!(instructions.calls.len(), 2);

        // Call[0]: IERC20.transfer(user, user_amount)
        let user_transfer = IERC20::transferCall {
            to: user_address,
            value: expected_user_amount,
        }
        .abi_encode();

        assert_eq!(instructions.calls[0].target, output_token_evm);
        assert_eq!(instructions.calls[0].callData, Bytes::from(user_transfer));
        assert_eq!(instructions.calls[0].value, U256::from(0));

        // Call[1]: IERC20.transfer(fee_address, fee_amount)
        let fee_address = Address::from_str(fee_address_str).unwrap();
        let fee_transfer = IERC20::transferCall {
            to: fee_address,
            value: expected_fee,
        }
        .abi_encode();

        assert_eq!(instructions.calls[1].target, output_token_evm);
        assert_eq!(instructions.calls[1].callData, Bytes::from(fee_transfer));
        assert_eq!(instructions.calls[1].value, U256::from(0));
    }

    #[tokio::test]
    async fn test_realy_data_recipient_destination() {
        // Mock EVM gas estimation response and verify recipient selection
        let across = Across::default();
        let amount = U256::from(12345u64);
        let input_asset = AssetId::from(Chain::Ethereum, None);
        let output_token = Across::decode_address_bytes32(&Address::from_str("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85").unwrap()); // OP USDC
        let wallet_address = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();
        let deployment = AcrossDeployment::deployment_by_chain(&Chain::Optimism).unwrap();

        // Build a mock provider that always returns a valid JSON-RPC result for gas estimation
        let provider = Arc::new(AlienProviderMock {
            response: crate::network::mock::MockFn(Box::new(|_target| {
                // JsonRpcResult<String> matching client.request expectation
                "{\"id\":1,\"result\":\"0x5208\"}".to_string() // 21000
            })),
            timeout: Duration::from_millis(50),
        });

        // Case 1: empty message -> recipient is user address
        let (gas_limit, v3_relay_data) = across
            .estimate_gas_limit(
                &amount,
                true,
                &input_asset,
                output_token,
                &wallet_address,
                &[],
                &deployment,
                provider.clone(),
                Chain::Optimism,
            )
            .await
            .unwrap();

        assert_eq!(gas_limit, U256::from(21000u64));

        let expected_recipient_user = Across::decode_address_bytes32(&wallet_address);

        assert_eq!(v3_relay_data.recipient, expected_recipient_user);

        // Assert tokens for ETH (native) -> OP USDC
        let expected_input_token = Across::decode_address_bytes32(&Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());

        assert_eq!(v3_relay_data.inputToken, expected_input_token);
        assert_eq!(v3_relay_data.outputToken, output_token);

        // Case 2: non-empty message -> recipient is multicall handler address
        let message = vec![0x01];
        let (gas_limit2, v3_relay_data2) = across
            .estimate_gas_limit(
                &amount,
                true,
                &input_asset,
                output_token,
                &wallet_address,
                &message,
                &deployment,
                provider.clone(),
                Chain::Optimism,
            )
            .await
            .unwrap();

        assert_eq!(gas_limit2, U256::from(21000u64));

        let multicall_addr = Address::from_str(deployment.multicall_handler().as_str()).unwrap();
        let expected_recipient_mc = Across::decode_address_bytes32(&multicall_addr);

        assert_eq!(v3_relay_data2.recipient, expected_recipient_mc);
        assert_eq!(v3_relay_data2.inputToken, expected_input_token);
        assert_eq!(v3_relay_data2.outputToken, output_token);

        // Case 3: ETH (Ethereum) -> ETH (Base) should use WETH addresses for input/output tokens
        let base_deployment = AcrossDeployment::deployment_by_chain(&Chain::Base).unwrap();
        let base_weth = "0x4200000000000000000000000000000000000006";
        let output_token_base = Across::decode_address_bytes32(&Address::from_str(base_weth).unwrap());
        let (gas_limit3, v3_relay_data3) = across
            .estimate_gas_limit(
                &amount,
                true,
                &input_asset,      // native ETH on Ethereum
                output_token_base, // WETH on Base
                &wallet_address,
                &[],
                &base_deployment,
                provider,
                Chain::Base,
            )
            .await
            .unwrap();

        assert_eq!(gas_limit3, U256::from(21000u64));

        let expected_input_token_eth_weth = Across::decode_address_bytes32(&Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());
        let expected_output_token_base_weth = Across::decode_address_bytes32(&Address::from_str(base_weth).unwrap());

        assert_eq!(v3_relay_data3.inputToken, expected_input_token_eth_weth);
        assert_eq!(v3_relay_data3.outputToken, expected_output_token_base_weth);
    }
}
