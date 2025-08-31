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
    Address, Bytes, U256,
};
use alloy_sol_types::{SolCall, SolValue};
use bs58;

use async_trait::async_trait;
use gem_evm::{
    across::{
        contracts::{
            multicall_handler,
            V3SpokePoolInterface::{self, V3RelayData},
        },
        deployment::AcrossDeployment,
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    erc20::IERC20,
    jsonrpc::TransactionObject,
    weth::WETH9,
};
use num_bigint::{BigInt, Sign};
use primitives::{swap::SwapStatus, AssetId, Chain, EVMChain};
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug, Clone)]
enum AddressType {
    Evm(Address),
    Solana(String),
}

impl AddressType {
    fn to_bytes32(&self) -> Result<[u8; 32], SwapperError> {
        match self {
            AddressType::Evm(address) => {
                // EVM address padded with zeros to 32 bytes
                let mut bytes32 = [0u8; 32];
                bytes32[12..32].copy_from_slice(address.as_slice()); // Address is 20 bytes, starts at byte 12
                Ok(bytes32)
            }
            AddressType::Solana(address_str) => {
                // Solana address - full 32 bytes
                Self::solana_address_to_bytes32(address_str)
            }
        }
    }

    fn solana_address_to_bytes32(solana_address: &str) -> Result<[u8; 32], SwapperError> {
        let decoded = bs58::decode(solana_address).into_vec().map_err(|_| SwapperError::InvalidAddress(solana_address.to_string()))?;

        if decoded.len() != 32 {
            return Err(SwapperError::InvalidAddress(solana_address.to_string()));
        }

        decoded.try_into().map_err(|_| SwapperError::InvalidAddress(solana_address.to_string()))
    }
}

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
        // Check if this is Solana to EVM (not supported yet)
        if from_asset.chain == Chain::Solana {
            return false;
        }

        if to_asset.chain == Chain::Solana {
            // Only support USDC for now
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

        // Handle EVM to EVM pairs (existing logic)
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

    fn get_address_type_for_chain(chain: Chain, token_id: Option<&str>) -> AddressType {
        match chain {
            Chain::Solana => AddressType::Solana(token_id.expect("token_id must be present for Solana assets").to_string()),
            _ => {
                // For EVM chains, parse the token_id as Address
                let address = if let Some(id) = token_id {
                    Address::from_str(id).unwrap_or(Address::ZERO)
                } else {
                    Address::ZERO
                };
                AddressType::Evm(address)
            }
        }
    }

    // Decode V3RelayData with proper cross-chain address handling
    fn decode_v3_relay_data(encoded_data: &[u8], request: &SwapperQuoteRequest) -> Result<V3RelayData, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        
        // If no Solana involved, use standard decoding
        if from_chain != Chain::Solana && to_chain != Chain::Solana {
            return V3RelayData::abi_decode(encoded_data).map_err(|_| SwapperError::InvalidRoute);
        }
        
        // For cross-chain with Solana, we need to fix the addresses before decoding
        let mut decode_data = encoded_data.to_vec();
        
        // V3RelayData struct fields in order:
        // 0: depositor (32 bytes)
        // 1: recipient (32 bytes) 
        // 2: exclusiveRelayer (32 bytes)
        // 3: inputToken (32 bytes) <- fix if from Solana
        // 4: outputToken (32 bytes) <- fix if to Solana
        let input_token_offset = 3 * 32;
        let output_token_offset = 4 * 32;
        
        // Convert Solana addresses back to EVM-compatible format for decoding
        // This creates placeholder EVM addresses that will be replaced in the actual transaction
        
        // Fix inputToken if from Solana  
        if from_chain == Chain::Solana && decode_data.len() >= input_token_offset + 32 {
            // Replace with zero address for decoding compatibility
            let zero_addr_bytes = [0u8; 32];
            decode_data[input_token_offset..input_token_offset + 32].copy_from_slice(&zero_addr_bytes);
        }
        
        // Fix outputToken if to Solana
        if to_chain == Chain::Solana && decode_data.len() >= output_token_offset + 32 {
            // Replace with zero address for decoding compatibility  
            let zero_addr_bytes = [0u8; 32];
            decode_data[output_token_offset..output_token_offset + 32].copy_from_slice(&zero_addr_bytes);
        }
        
        V3RelayData::abi_decode(&decode_data).map_err(|_| SwapperError::InvalidRoute)
    }

    fn encode_v3_relay_data(relay_data: &V3RelayData, request: &SwapperQuoteRequest) -> Result<Vec<u8>, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();

        // If no Solana involved, use standard encoding
        if from_chain != Chain::Solana && to_chain != Chain::Solana {
            return Ok(relay_data.abi_encode());
        }

        // Start with standard encoding and fix the addresses
        let mut encoded_data = relay_data.abi_encode();

        // V3RelayData struct fields in order:
        // 0: depositor (32 bytes)
        // 1: recipient (32 bytes)
        // 2: exclusiveRelayer (32 bytes)
        // 3: inputToken (32 bytes) <- fix if from Solana
        // 4: outputToken (32 bytes) <- fix if to Solana
        let input_token_offset = 3 * 32;
        let output_token_offset = 4 * 32;

        // Fix inputToken if from Solana
        if from_chain == Chain::Solana && encoded_data.len() >= input_token_offset + 32 {
            let from_asset_id = request.from_asset.asset_id();
            let input_addr_type = Self::get_address_type_for_chain(from_chain, from_asset_id.token_id.as_deref());
            let input_bytes = input_addr_type.to_bytes32()?;
            encoded_data[input_token_offset..input_token_offset + 32].copy_from_slice(&input_bytes);
        }

        // Fix outputToken if to Solana
        if to_chain == Chain::Solana && encoded_data.len() >= output_token_offset + 32 {
            let to_asset_id = request.to_asset.asset_id();
            let output_addr_type = Self::get_address_type_for_chain(to_chain, to_asset_id.token_id.as_deref());
            let output_bytes = output_addr_type.to_bytes32()?;
            encoded_data[output_token_offset..output_token_offset + 32].copy_from_slice(&output_bytes);
        }

        Ok(encoded_data)
    }

    fn is_solana_destination(request: &SwapperQuoteRequest) -> bool {
        request.to_asset.chain() == Chain::Solana
    }

    fn get_output_asset_and_token(request: &SwapperQuoteRequest) -> Result<(AssetId, Address), SwapperError> {
        if Self::is_solana_destination(request) {
            // For Solana, keep original asset ID and use a placeholder address
            // The actual full 32-byte Solana address will be inserted during encoding
            let solana_output_asset = request.to_asset.asset_id();
            let placeholder_address = Address::ZERO; // Placeholder, gets replaced in fix_v3_relay_data_encoding_for_solana
            Ok((solana_output_asset, placeholder_address))
        } else {
            let norm_output_asset = eth_address::normalize_weth_asset(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedPair)?;
            let output_token = eth_address::parse_asset_id(&norm_output_asset)?;
            Ok((norm_output_asset, output_token))
        }
    }

    fn get_destination_chain_id(chain: &Chain) -> Result<u32, SwapperError> {
        let deployment = AcrossDeployment::deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        deployment.chain_id.try_into().map_err(|_| SwapperError::NotSupportedChain)
    }

    fn calculate_relayer_fee_for_destination(
        request: &SwapperQuoteRequest,
        from_amount: U256,
        cost_config: &fees::CapitalCostConfig,
        sol_price: Option<&BigInt>,
    ) -> U256 {
        if Self::is_solana_destination(request) {
            // Calculate Solana relayer fee: 0.000005 SOL converted to USDC using price feed
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

    fn get_gas_chain_for_estimation(&self, request: &SwapperQuoteRequest) -> Chain {
        if Self::is_solana_destination(request) {
            // For Solana destination, use source chain for gas estimation
            request.from_asset.chain()
        } else {
            request.to_asset.chain()
        }
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
        output_token: &Address,
        wallet_address: &Address,
        message: &[u8],
        deployment: &AcrossDeployment,
        provider: Arc<dyn AlienProvider>,
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
        let gas_limit = eth_rpc::estimate_gas(provider, chain, tx).await;
        Ok((gas_limit.unwrap_or(U256::from(DEFAULT_FILL_GAS_LIMIT)), v3_relay_data))
    }

    pub fn update_v3_relay_data(
        &self,
        v3_relay_data: &mut V3RelayData,
        user_address: &Address,
        output_amount: &U256,
        original_output_asset: &AssetId,
        output_token: &Address,
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
        // does not support same chain swap
        if request.from_asset.chain() == request.to_asset.chain() {
            return Err(SwapperError::NotSupportedPair);
        }

        // Check for SOLANA -> EVM case (not supported yet)
        if request.from_asset.chain() == Chain::Solana {
            return Err(SwapperError::NotSupportedPair);
        }

        // Continue with EVM -> EVM logic
        let input_is_native = request.from_asset.is_native();
        let from_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(SwapperError::from)?;
        let wallet_address = eth_address::parse_str(&request.wallet_address)?;

        let _from_deployment = AcrossDeployment::deployment_by_chain(&request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_asset = eth_address::normalize_weth_asset(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedPair)?;
        let (output_asset, output_token) = Self::get_output_asset_and_token(request)?;
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

        // Calculate lp fee
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent, cost_config.decimals);
        debug_println!("lpfee: {}", lpfee);

        // Get SOL price if this is a Solana destination
        let sol_price = if let Some(index) = sol_price_index {
            Some(ChainlinkPriceFeed::decoded_answer(&multicall_results[index])?)
        } else {
            None
        };

        // Calculate relayer fee
        let relayer_fee = Self::calculate_relayer_fee_for_destination(request, from_amount, cost_config, sol_price.as_ref());
        debug_println!("relayer_fee: {}", relayer_fee);

        let referral_config = request.options.fee.clone().unwrap_or_default().evm_bridge;

        // Calculate gas limit / price for relayer
        let remain_amount = from_amount - lpfee - relayer_fee;
        let (message, referral_fee) =
            self.message_for_multicall_handler(&remain_amount, &original_output_asset, &wallet_address, &output_token, &referral_config);

        let gas_chain = self.get_gas_chain_for_estimation(request);
        let gas_price_req = eth_rpc::fetch_gas_price(provider.clone(), gas_chain);
        let gas_limit_req = self.estimate_gas_limit(
            &from_amount,
            input_is_native,
            &input_asset,
            &output_token,
            &wallet_address,
            &message,
            &destination_deployment,
            provider.clone(),
            gas_chain,
        );

        let (tuple, gas_price) = futures::join!(gas_limit_req, gas_price_req);
        let (gas_limit, mut v3_relay_data) = tuple?;
        let mut gas_fee = gas_limit * gas_price?;
        if let Some(index) = eth_price_index {
            let eth_price = ChainlinkPriceFeed::decoded_answer(&multicall_results[index])?;
            gas_fee = Self::calculate_fee_in_token(&gas_fee, &eth_price, 6);
        }
        debug_println!("gas_fee: {}", gas_fee);

        // Check if bridge amount is too small
        if remain_amount < gas_fee {
            return Err(SwapperError::InputAmountTooSmall);
        }

        let output_amount = remain_amount - gas_fee;
        let to_value = output_amount - referral_fee;

        // Update v3 relay data with final amounts
        self.update_v3_relay_data(
            &mut v3_relay_data,
            &wallet_address,
            &output_amount,
            &original_output_asset,
            &output_token,
            timestamp,
            &referral_config,
        )?;
        // Encode V3RelayData with proper cross-chain address handling
        let encoded_data = Self::encode_v3_relay_data(&v3_relay_data, request)?;
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
        let v3_relay_data = Self::decode_v3_relay_data(&route_data, &quote.request)?;

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

        let approval: Option<SwapperApprovalData> = {
            if input_is_native {
                None
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    v3_relay_data.inputToken.to_string(),
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
    use gem_evm::multicall3::IMulticall3;
    use primitives::asset_constants::*;

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

        // Test supported EVM -> Solana USDC pairs
        assert!(Across::is_supported_pair(&usdc_eth, &solana_usdc));
        assert!(Across::is_supported_pair(&usdc_arb, &solana_usdc));

        // Test unsupported Solana pairs
        let solana_usdt = AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
        assert!(!Across::is_supported_pair(&usdc_eth, &solana_usdt)); // Only USDC supported

        // Solana -> EVM pairs (not supported)
        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_eth));
        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_arb));

        // Non-USDC to Solana (not supported)
        assert!(!Across::is_supported_pair(&weth_eth, &solana_usdc));
    }

    #[test]
    fn test_solana_address_to_bytes32() {
        // Test USDC conversion to full 32 bytes
        let usdc_result = AddressType::solana_address_to_bytes32("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        assert!(usdc_result.is_ok());

        // Real hex data: USDC token address decoded from base58 (full 32 bytes)
        // EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v -> c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61
        let expected_bytes = HexDecode("c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61").unwrap();
        assert_eq!(usdc_result.unwrap().as_slice(), expected_bytes.as_slice());

        // Test USDT address - Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
        let usdt_result = AddressType::solana_address_to_bytes32("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
        assert!(usdt_result.is_ok());
        // Verify it produces valid 32-byte array
        let actual_usdt = usdt_result.unwrap();
        assert_eq!(actual_usdt.len(), 32);

        // Test invalid address
        let invalid_result = AddressType::solana_address_to_bytes32("invalid_address");
        assert!(invalid_result.is_err());

        // Test short address
        let short_result = AddressType::solana_address_to_bytes32("123");
        assert!(short_result.is_err());
    }

    #[test]
    fn test_address_type_enum() {
        // Test EVM address type
        let evm_addr = Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap();
        let evm_type = AddressType::Evm(evm_addr);
        let evm_bytes32 = evm_type.to_bytes32().unwrap();

        // Should be padded with zeros
        let mut expected_evm = [0u8; 32];
        expected_evm[12..32].copy_from_slice(evm_addr.as_slice());
        assert_eq!(evm_bytes32, expected_evm);

        // Test Solana address type
        let solana_type = AddressType::Solana("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());
        let solana_bytes32 = solana_type.to_bytes32().unwrap();

        // Should be full 32 bytes
        let expected_solana = HexDecode("c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61").unwrap();
        assert_eq!(solana_bytes32.as_slice(), expected_solana.as_slice());

        // Test get_address_type_for_chain
        let evm_chain_type = Across::get_address_type_for_chain(Chain::Ethereum, Some("0xaf88d065e77c8cc2239327c5edb3a432268e5831"));
        let solana_chain_type = Across::get_address_type_for_chain(Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));

        assert!(matches!(evm_chain_type, AddressType::Evm(_)));
        assert!(matches!(solana_chain_type, AddressType::Solana(_)));
    }

    #[test]
    fn test_v3_relay_data_solana_encoding() {
        use gem_evm::across::contracts::V3SpokePoolInterface::V3RelayData;
        use primitives::swap::{QuoteAsset, SwapMode};

        // Create a mock V3RelayData
        let input_token = Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap(); // USDC on Arbitrum
        let placeholder_output_addr = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let v3_relay_data = V3RelayData {
            depositor: Address::ZERO,
            recipient: Address::ZERO,
            exclusiveRelayer: Address::ZERO,
            inputToken: input_token,
            outputToken: placeholder_output_addr,
            inputAmount: U256::from(1000000u64), // 1 USDC
            outputAmount: U256::from(1000000u64),
            originChainId: U256::from(42161u64), // Arbitrum
            depositId: 12345,
            fillDeadline: 1700000000,
            exclusivityDeadline: 0,
            message: Bytes::new(),
        };

        // Standard encoding (what we get without the fix)
        let standard_encoded = v3_relay_data.abi_encode();

        // Create a mock SwapperQuoteRequest for Solana destination
        let solana_usdc = SOLANA_USDC.clone();
        let arb_usdc = ARBITRUM_USDC.clone();
        let request = SwapperQuoteRequest {
            from_asset: QuoteAsset {
                id: arb_usdc.id.to_string(),
                symbol: arb_usdc.symbol,
                decimals: arb_usdc.decimals as u32,
            },
            to_asset: QuoteAsset {
                id: solana_usdc.id.to_string(),
                symbol: solana_usdc.symbol,
                decimals: solana_usdc.decimals as u32,
            },
            value: "1000000".to_string(),
            wallet_address: "0x742d35Cc6634C0532925a3b8D0B4E4c7bf5389ef".to_string(),
            destination_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            mode: SwapMode::ExactIn,
            options: Default::default(),
        };

        // Apply the cross-chain encoding
        let fixed_encoded = Across::encode_v3_relay_data(&v3_relay_data, &request).unwrap();

        // The fixed encoding should be different from the standard encoding for Solana destinations
        assert_ne!(standard_encoded, fixed_encoded, "Fixed encoding should differ from standard encoding");

        // Verify that the fixed encoding contains the full Solana address at the expected location
        let output_token_offset = 4 * 32; // 4th field (0-indexed)
        let expected_full_solana_bytes = AddressType::solana_address_to_bytes32("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        assert_eq!(
            &fixed_encoded[output_token_offset..output_token_offset + 32],
            &expected_full_solana_bytes[..],
            "Fixed encoding should contain full 32-byte Solana address at outputToken position"
        );
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
