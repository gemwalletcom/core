use super::{
    DEFAULT_FILL_TIMEOUT,
    api::AcrossApi,
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
    models::{DestinationMessage, QuoteContext, RelayRecipient},
    solana::{AcrossPlusMessage, CompiledIx, DEFAULT_SOLANA_COMPUTE_LIMIT, MULTICALL_HANDLER, SOL_NATIVE_DECIMALS},
    solana_tx,
};
use crate::{
    SwapResult, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    across::{DEFAULT_DEPOSIT_GAS_LIMIT, DEFAULT_FILL_GAS_LIMIT, MESSAGE_GAS_MULTIPLIER},
    alien::RpcProvider,
    approval::check_approval_erc20,
    asset::*,
    chainlink::ChainlinkPriceFeed,
    client_factory::{create_client_with_chain, create_eth_client},
    error::{INVALID_ADDRESS, INVALID_AMOUNT},
    eth_address,
    models::*,
};
use alloy_primitives::{
    Address, Bytes, FixedBytes, U256,
    hex::{decode as HexDecode, encode_prefixed as HexEncode},
};
use alloy_sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use bs58;
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
use gem_solana::{jsonrpc::SolanaRpc, models::prioritization_fee::SolanaPrioritizationFee};
use num_bigint::{BigInt, Sign};
use primitives::{AssetId, Chain, ChainType, EVMChain, swap::ApprovalData, swap::SwapStatus};
use serde_serializers::biguint_from_hex_str;
use solana_primitives::{
    instructions::{associated_token::get_associated_token_address, program_ids, token::transfer as spl_transfer},
    types::{Instruction as SolInstruction, Pubkey as SolanaPubkey, find_program_address},
};
use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::Arc};

struct PoolState {
    token_config: TokenConfig,
    utilization_before: BigInt,
    utilization_after: BigInt,
    timestamp: u32,
    eth_price: Option<BigInt>,
    sol_price: Option<BigInt>,
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

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        if from_asset.chain == Chain::Solana || to_asset.chain == Chain::Solana {
            return false;
        }

        let Some(from) = eth_address::convert_native_to_weth(from_asset) else {
            return false;
        };
        let Some(to) = eth_address::convert_native_to_weth(to_asset) else {
            return false;
        };

        AcrossDeployment::asset_mappings().into_iter().any(|x| x.set.contains(&from) && x.set.contains(&to))
    }

    fn decode_address_bytes32(addr: &Address) -> FixedBytes<32> {
        let mut bytes = [0u8; 32];
        bytes[12..32].copy_from_slice(addr.as_slice());
        FixedBytes::from(bytes)
    }

    fn decode_bs58_bytes32(addr: &str) -> Result<FixedBytes<32>, SwapperError> {
        let invalid_address = || SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {addr}"));
        let decoded = bs58::decode(addr).into_vec().map_err(|_| invalid_address())?;
        if decoded.len() != 32 {
            return Err(invalid_address());
        }
        let bytes: [u8; 32] = decoded.try_into().map_err(|_| invalid_address())?;
        Ok(FixedBytes::from(bytes))
    }

    fn recipient_to_fixed_bytes(recipient: &RelayRecipient) -> Result<FixedBytes<32>, SwapperError> {
        match recipient {
            RelayRecipient::Evm(address) => Ok(Self::decode_address_bytes32(address)),
            RelayRecipient::Solana(pubkey) => Ok(FixedBytes::from(*pubkey.as_bytes())),
        }
    }

    fn recipient_evm_address(recipient: &RelayRecipient) -> Option<&Address> {
        match recipient {
            RelayRecipient::Evm(address) => Some(address),
            RelayRecipient::Solana(_) => None,
        }
    }

    fn token_bytes32_for_asset(asset: &AssetId) -> Result<FixedBytes<32>, SwapperError> {
        match asset.chain.chain_type() {
            ChainType::Solana => {
                let id = asset
                    .token_id
                    .as_deref()
                    .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: missing token_id for Solana")))?;
                Self::decode_bs58_bytes32(id)
            }
            ChainType::Ethereum => {
                let evm_chain = EVMChain::from_chain(asset.chain).ok_or(SwapperError::NotSupportedChain)?;
                let default_weth = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;
                let id = if asset.is_native() { default_weth } else { asset.token_id.as_deref().unwrap() };
                Ok(Self::decode_address_bytes32(&eth_address::parse_str(id)?))
            }
            _ => Err(SwapperError::NotSupportedChain),
        }
    }

    fn is_solana_destination(request: &QuoteRequest) -> bool {
        request.to_asset.chain() == Chain::Solana
    }

    fn is_solana_origin(request: &QuoteRequest) -> bool {
        request.from_asset.chain() == Chain::Solana
    }

    fn get_output_asset(request: &QuoteRequest) -> Result<AssetId, SwapperError> {
        if Self::is_solana_destination(request) {
            Ok(request.to_asset.asset_id())
        } else {
            eth_address::convert_native_to_weth(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)
        }
    }

    fn get_destination_chain_id(chain: &Chain) -> Result<u64, SwapperError> {
        let deployment = AcrossDeployment::deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        Ok(deployment.chain_id)
    }

    fn build_context<'a>(&self, request: &'a QuoteRequest) -> Result<QuoteContext<'a>, SwapperError> {
        if request.from_asset.chain() == request.to_asset.chain() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        if request.from_asset.chain() == Chain::Solana || request.to_asset.chain() == Chain::Solana {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let from_amount: U256 = request.value.parse().map_err(SwapperError::from)?;
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let is_solana_origin = Self::is_solana_origin(request);
        let depositor = if is_solana_origin {
            let depositor_address =
                SolanaPubkey::from_str(&request.wallet_address).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {}", request.wallet_address)))?;
            RelayRecipient::Solana(depositor_address)
        } else {
            RelayRecipient::Evm(eth_address::parse_str(&request.wallet_address)?)
        };
        let evm_address = if is_solana_origin {
            eth_address::parse_str(&request.destination_address)?
        } else {
            eth_address::parse_str(&request.wallet_address)?
        };

        let _origin_deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&to_chain).ok_or(SwapperError::NotSupportedChain)?;

        if !Self::is_supported_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_asset = if is_solana_origin {
            request.from_asset.asset_id()
        } else {
            eth_address::convert_native_to_weth(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?
        };
        let output_asset = Self::get_output_asset(request)?;
        let original_output_asset = request.to_asset.asset_id();

        let asset_mapping = AcrossDeployment::asset_mappings()
            .into_iter()
            .find(|mapping| mapping.set.contains(&input_asset))
            .ok_or(SwapperError::NoQuoteAvailable)?;
        let mainnet_asset = asset_mapping
            .set
            .iter()
            .find(|asset| asset.chain == Chain::Ethereum)
            .cloned()
            .ok_or(SwapperError::NoQuoteAvailable)?;
        let mainnet_chain = EVMChain::from_chain(mainnet_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let mainnet_token = eth_address::parse_or_weth_address(&mainnet_asset, mainnet_chain)?;

        let referral_fees = request.options.fee.clone().unwrap_or_default();
        let referral_fee = referral_fees.evm_bridge;

        let output_token_decimals = u8::try_from(asset_mapping.capital_cost.decimals).map_err(|_| SwapperError::ComputeQuoteError("Unsupported token decimals".into()))?;

        Ok(QuoteContext {
            from_amount,
            depositor,
            evm_address,
            from_chain,
            to_chain,
            input_is_native: request.from_asset.is_native(),
            input_asset,
            output_asset,
            original_output_asset,
            mainnet_token,
            capital_cost: asset_mapping.capital_cost,
            referral_fee,
            destination_deployment,
            solana_destination_address: if to_chain == Chain::Solana { Some(request.destination_address.as_str()) } else { None },
            output_token_decimals,
        })
    }

    async fn fetch_pool_state(&self, ctx: &QuoteContext<'_>) -> Result<PoolState, SwapperError> {
        let hubpool_client = HubPoolClient::new(self.rpc_provider.clone(), Chain::Ethereum);
        let config_client = ConfigStoreClient::new(self.rpc_provider.clone(), Chain::Ethereum);

        let preflight_calls = vec![
            hubpool_client.paused_call3(),
            hubpool_client.sync_call3(&ctx.mainnet_token),
            hubpool_client.pooled_token_call3(&ctx.mainnet_token),
        ];
        let preflight_results = self.multicall3(hubpool_client.chain, preflight_calls).await?;

        if hubpool_client.decoded_paused_call3(&preflight_results[0])? {
            return Err(SwapperError::ComputeQuoteError("Across protocol is paused".into()));
        }

        let reserves = hubpool_client.decoded_pooled_token_call3(&preflight_results[2])?.liquidReserves;
        if ctx.from_amount > reserves {
            return Err(SwapperError::ComputeQuoteError("Bridge amount is too large".into()));
        }

        let token_config_future = config_client.fetch_config(&ctx.mainnet_token);

        let mut call_requests = vec![
            hubpool_client.utilization_call3(&ctx.mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&ctx.mainnet_token, ctx.from_amount),
            hubpool_client.get_current_time(),
        ];

        let mut index_tracker: HashMap<&'static str, usize> = HashMap::new();
        let mut next_index = 3usize;

        if !ctx.input_is_native && ctx.to_chain != Chain::Monad {
            let feed = ChainlinkPriceFeed::new_usd_feed_for_chain(ctx.to_chain).unwrap_or_else(ChainlinkPriceFeed::new_eth_usd_feed);
            call_requests.push(feed.latest_round_call3());
            index_tracker.insert("eth_price", next_index);
            next_index += 1;
        }

        if ctx.to_chain == Chain::Solana {
            call_requests.push(ChainlinkPriceFeed::new_sol_usd_feed().latest_round_call3());
            index_tracker.insert("sol_price", next_index);
        }

        let multicall_future = self.multicall3(hubpool_client.chain, call_requests);
        let (token_config, multicall_results) = futures::join!(token_config_future, multicall_future);

        let token_config = token_config?;
        let multicall_results = multicall_results?;

        let utilization_before = hubpool_client.decoded_utilization_call3(&multicall_results[0])?;
        let utilization_after = hubpool_client.decoded_utilization_call3(&multicall_results[1])?;
        let timestamp = hubpool_client.decoded_current_time(&multicall_results[2])?;

        let mut eth_price = None;
        if !ctx.input_is_native {
            if ctx.to_chain == Chain::Monad {
                let feed = ChainlinkPriceFeed::new_usd_feed_for_chain(ctx.to_chain).unwrap_or_else(ChainlinkPriceFeed::new_eth_usd_feed);
                let results = create_eth_client(self.rpc_provider.clone(), Chain::Monad)?
                    .multicall3(vec![feed.latest_round_call3()])
                    .await
                    .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;
                eth_price = Some(ChainlinkPriceFeed::decoded_answer(&results[0])?);
            } else if let Some(index) = index_tracker.get("eth_price") {
                eth_price = Some(ChainlinkPriceFeed::decoded_answer(&multicall_results[*index])?);
            }
        }

        let sol_price = index_tracker
            .get("sol_price")
            .map(|index| ChainlinkPriceFeed::decoded_answer(&multicall_results[*index]))
            .transpose()?;

        Ok(PoolState {
            token_config,
            utilization_before,
            utilization_after,
            timestamp,
            eth_price,
            sol_price,
        })
    }

    fn build_v3_relay_data(&self, ctx: &QuoteContext<'_>, recipient: FixedBytes<32>, output_token: FixedBytes<32>, message: &[u8]) -> Result<V3RelayData, SwapperError> {
        let origin_chain_id = Self::get_destination_chain_id(&ctx.from_chain)?;
        let depositor = Self::recipient_to_fixed_bytes(&ctx.depositor)?;

        Ok(V3RelayData {
            depositor,
            recipient,
            exclusiveRelayer: FixedBytes::from([0u8; 32]),
            inputToken: Self::token_bytes32_for_asset(&ctx.input_asset)?,
            outputToken: output_token,
            inputAmount: ctx.from_amount,
            outputAmount: U256::from(100),
            originChainId: U256::from(origin_chain_id),
            depositId: U256::from(u32::MAX),
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(message.to_vec()),
        })
    }

    fn calculate_relayer_capital_fee(from_amount: U256, cost_config: &fees::CapitalCostConfig) -> U256 {
        let relayer_calc = RelayerFeeCalculator::default();
        let from_amount_bigint = BigInt::from_bytes_le(Sign::Plus, &from_amount.to_le_bytes::<32>());
        let relayer_fee_percent = relayer_calc.capital_fee_percent(&from_amount_bigint, cost_config);
        fees::multiply(from_amount, relayer_fee_percent, cost_config.decimals)
    }

    pub fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    fn build_destination_message(&self, ctx: &QuoteContext<'_>, amount: &U256, output_token_evm: Option<&Address>) -> Result<DestinationMessage, SwapperError> {
        match ctx.to_chain.chain_type() {
            ChainType::Ethereum => self.build_evm_destination_message(ctx, amount, output_token_evm),
            ChainType::Solana => self.build_solana_destination_message(ctx, amount),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }

    fn build_evm_destination_message(&self, ctx: &QuoteContext<'_>, amount: &U256, output_token_evm: Option<&Address>) -> Result<DestinationMessage, SwapperError> {
        let referral_fee = &ctx.referral_fee;
        if referral_fee.bps == 0 || referral_fee.address.is_empty() {
            return Ok(DestinationMessage {
                bytes: vec![],
                referral_fee: U256::from(0),
                recipient: RelayRecipient::Evm(ctx.evm_address),
            });
        }

        let token = output_token_evm.ok_or(SwapperError::NotSupportedAsset)?;
        let fee_address = Address::from_str(&referral_fee.address).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {}", referral_fee.address)))?;
        let fee_amount = amount * U256::from(referral_fee.bps) / U256::from(10000);

        let calls = if ctx.original_output_asset.is_native() {
            Self::unwrap_weth_calls(token, amount, &fee_address, &fee_amount)
        } else {
            Self::erc20_transfer_calls(token, &fee_address, &fee_amount)
        };

        let instructions = multicall_handler::Instructions {
            calls,
            fallbackRecipient: ctx.evm_address,
        };
        let message = instructions.abi_encode();
        let multicall_address = eth_address::parse_str(ctx.destination_deployment.multicall_handler().as_str())?;

        Ok(DestinationMessage {
            bytes: message,
            referral_fee: fee_amount,
            recipient: RelayRecipient::Evm(multicall_address),
        })
    }

    fn build_solana_destination_message(&self, ctx: &QuoteContext<'_>, amount: &U256) -> Result<DestinationMessage, SwapperError> {
        let destination_address = ctx
            .solana_destination_address
            .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: Missing Solana destination address")))?;
        let user_account = SolanaPubkey::from_str(destination_address).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {destination_address}")))?;

        let referral_fee = &ctx.referral_fee;
        if referral_fee.bps == 0 || referral_fee.address.is_empty() {
            return Ok(DestinationMessage {
                bytes: vec![],
                referral_fee: U256::from(0),
                recipient: RelayRecipient::Solana(user_account),
            });
        }

        let referral_account =
            SolanaPubkey::from_str(&referral_fee.address).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {}", referral_fee.address)))?;
        let handler_program = SolanaPubkey::from_str(MULTICALL_HANDLER).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {MULTICALL_HANDLER}")))?;
        let (handler_signer, _) =
            find_program_address(&handler_program, &[b"handler_signer"]).map_err(|_| SwapperError::ComputeQuoteError("Failed to derive handler signer".into()))?;

        let mint_id = ctx
            .original_output_asset
            .token_id
            .as_deref()
            .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: Missing Solana mint")))?;
        let mint = SolanaPubkey::from_str(mint_id).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {mint_id}")))?;

        let token_program =
            SolanaPubkey::from_str(program_ids::TOKEN_PROGRAM_ID).map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_ADDRESS}: {}", program_ids::TOKEN_PROGRAM_ID)))?;

        let handler_token_account = get_associated_token_address(&handler_signer, &mint);
        let referral_token_account = get_associated_token_address(&referral_account, &mint);
        let user_token_account = get_associated_token_address(&user_account, &mint);

        let fee_amount = amount * U256::from(referral_fee.bps) / U256::from(10000);
        let user_amount = amount - fee_amount;

        let fee_amount_u64: u64 = fee_amount
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_AMOUNT}: Referral fee overflow")))?;
        let user_amount_u64: u64 = user_amount
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError(format!("{INVALID_AMOUNT}: User amount overflow")))?;

        let transfer_fee_ix = spl_transfer(&handler_token_account, &referral_token_account, &handler_signer, fee_amount_u64);
        let transfer_user_ix = spl_transfer(&handler_token_account, &user_token_account, &handler_signer, user_amount_u64);

        let accounts = vec![handler_token_account, referral_token_account, user_token_account, handler_signer, token_program];

        let compiled_ixs = self.compile_solana_instructions(&[transfer_fee_ix, transfer_user_ix], &accounts)?;
        let handler_message = borsh::to_vec(&compiled_ixs).map_err(|_| SwapperError::ComputeQuoteError("Failed to encode handler message".into()))?;

        let across_message = AcrossPlusMessage {
            handler: handler_program,
            read_only_len: 2, // handler_signer and token_program are read-only
            value_amount: 0,
            accounts,
            handler_message,
        };
        let message_bytes = borsh::to_vec(&across_message).map_err(|_| SwapperError::ComputeQuoteError("Failed to encode Across message".into()))?;

        Ok(DestinationMessage {
            bytes: message_bytes,
            referral_fee: fee_amount,
            recipient: RelayRecipient::Solana(handler_signer),
        })
    }

    fn compile_solana_instructions(&self, instructions: &[SolInstruction], accounts: &[SolanaPubkey]) -> Result<Vec<CompiledIx>, SwapperError> {
        let mut account_index_map: HashMap<String, u8> = HashMap::new();
        for (idx, account) in accounts.iter().enumerate() {
            account_index_map.insert(account.to_base58(), idx as u8);
        }

        let mut compiled = Vec::with_capacity(instructions.len());
        for instruction in instructions {
            let program_key = instruction.program_id.to_base58();
            let program_index = account_index_map
                .get(&program_key)
                .copied()
                .ok_or_else(|| SwapperError::ComputeQuoteError("Program account missing from message".into()))?;

            let mut account_key_indexes = Vec::with_capacity(instruction.accounts.len());
            for account in &instruction.accounts {
                let key = account.pubkey.to_base58();
                let index = account_index_map
                    .get(&key)
                    .copied()
                    .ok_or_else(|| SwapperError::ComputeQuoteError("Account missing from message".into()))?;
                account_key_indexes.push(index);
            }

            compiled.push(CompiledIx {
                program_id_index: program_index,
                account_key_indexes,
                data: instruction.data.clone(),
            });
        }

        Ok(compiled)
    }

    fn unwrap_weth_calls(weth_contract: &Address, output_amount: &U256, fee_address: &Address, fee_amount: &U256) -> Vec<multicall_handler::Call> {
        assert!(*fee_amount <= *output_amount);
        let withdraw_call = WETH9::withdrawCall { wad: *output_amount };
        vec![
            multicall_handler::Call {
                target: *weth_contract,
                callData: withdraw_call.abi_encode().into(),
                value: U256::from(0),
            },
            multicall_handler::Call {
                target: *fee_address,
                callData: Bytes::new(),
                value: *fee_amount,
            },
        ]
    }

    fn erc20_transfer_calls(token: &Address, fee_address: &Address, fee_amount: &U256) -> Vec<multicall_handler::Call> {
        let target = *token;
        let fee_transfer = IERC20::transferCall {
            to: *fee_address,
            value: *fee_amount,
        };
        vec![multicall_handler::Call {
            target,
            callData: fee_transfer.abi_encode().into(),
            value: U256::from(0),
        }]
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

    async fn estimate_gas_limit(
        &self,
        ctx: &QuoteContext<'_>,
        destination_message: &DestinationMessage,
        output_token: FixedBytes<32>,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain = ctx.to_chain;
        if chain.chain_type() != ChainType::Ethereum {
            return Err(SwapperError::NotSupportedChain);
        }

        let recipient_address = Self::recipient_evm_address(&destination_message.recipient).ok_or(SwapperError::NotSupportedChain)?;
        let recipient = Self::decode_address_bytes32(recipient_address);
        let v3_relay_data = self.build_v3_relay_data(ctx, recipient, output_token, &destination_message.bytes)?;

        let value = if ctx.input_is_native { format!("{:#x}", ctx.from_amount) } else { String::from("0x0") };
        let chain_id = Self::get_destination_chain_id(&chain)?;
        let data = V3SpokePoolInterface::fillRelayCall {
            relayData: v3_relay_data.clone(),
            repaymentChainId: U256::from(chain_id),
            repaymentAddress: Self::decode_address_bytes32(&ctx.evm_address),
        }
        .abi_encode();

        let tx = TransactionObject::new_call_to_value(ctx.destination_deployment.spoke_pool, &value, data);
        let gas_limit = self
            .estimate_gas_transaction(chain, tx)
            .await
            .unwrap_or_else(|_| U256::from(Self::get_default_fill_limit(chain)));
        Ok((gas_limit, v3_relay_data))
    }

    fn get_default_fill_limit(chain: Chain) -> u64 {
        match chain {
            Chain::Monad => DEFAULT_FILL_GAS_LIMIT * 3,
            _ => DEFAULT_FILL_GAS_LIMIT,
        }
    }

    fn update_v3_relay_data(&self, v3_relay_data: &mut V3RelayData, output_amount: &U256, timestamp: u32, destination_message: DestinationMessage) -> U256 {
        v3_relay_data.outputAmount = *output_amount;
        v3_relay_data.fillDeadline = timestamp + DEFAULT_FILL_TIMEOUT;
        v3_relay_data.message = destination_message.bytes.into();

        destination_message.referral_fee
    }

    pub fn calculate_fee_in_token(fee_in_wei: &U256, token_price: &BigInt, token_decimals: u32) -> U256 {
        Self::calculate_fee_in_token_with_native_decimals(fee_in_wei, token_price, token_decimals, 18)
    }

    fn calculate_fee_in_token_with_native_decimals(fee_in_native: &U256, token_price: &BigInt, token_decimals: u32, native_decimals: u32) -> U256 {
        let fee = BigInt::from_bytes_le(Sign::Plus, &fee_in_native.to_le_bytes::<32>());
        let fee_in_token = fee * token_price * BigInt::from(10_u64.pow(token_decimals)) / BigInt::from(10_u64.pow(8)) / BigInt::from(10_u64.pow(native_decimals));
        U256::from_le_slice(&fee_in_token.to_bytes_le().1)
    }

    async fn fetch_solana_unit_price(provider: Arc<dyn RpcProvider>) -> Result<u64, SwapperError> {
        let client = create_client_with_chain(provider, Chain::Solana);
        let rpc_call = SolanaRpc::GetRecentPrioritizationFees;
        let fees: Vec<SolanaPrioritizationFee> = client.request(rpc_call).await?;

        if fees.is_empty() {
            return Err(SwapperError::ComputeQuoteError("Failed to fetch recent prioritization fees".to_string()));
        }

        let total_fee: u64 = fees.iter().map(|f| f.prioritization_fee as u64).sum();
        let average_fee = total_fee / fees.len() as u64;

        Ok(std::cmp::max(1, average_fee))
    }

    async fn calculate_gas_price_and_fee(
        &self,
        ctx: &QuoteContext<'_>,
        destination_message: &DestinationMessage,
        output_token: FixedBytes<32>,
        eth_price: Option<&BigInt>,
        sol_price: Option<&BigInt>,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let has_message = !destination_message.bytes.is_empty();

        if ctx.to_chain == Chain::Solana {
            let unit_price = Self::fetch_solana_unit_price(self.rpc_provider.clone()).await?;
            let gas_fee_micro_lamports = DEFAULT_SOLANA_COMPUTE_LIMIT * unit_price;
            let gas_fee_lamports = gas_fee_micro_lamports / 1_000_000;
            let total_gas_lamports = gas_fee_lamports + 5000;

            let mut gas_fee = if let Some(price) = sol_price {
                Self::calculate_fee_in_token_with_native_decimals(&U256::from(total_gas_lamports), price, ctx.output_token_decimals as u32, SOL_NATIVE_DECIMALS)
            } else {
                U256::ZERO
            };

            if has_message {
                gas_fee *= U256::from(MESSAGE_GAS_MULTIPLIER);
            }

            let recipient = Self::recipient_to_fixed_bytes(&destination_message.recipient)?;
            let v3_relay_data = self.build_v3_relay_data(ctx, recipient, output_token, &destination_message.bytes)?;

            Ok((gas_fee, v3_relay_data))
        } else {
            let gas_chain = ctx.to_chain;
            let gas_price_req = self.gas_price(gas_chain);
            let gas_limit_req = self.estimate_gas_limit(ctx, destination_message, output_token);

            let (tuple, gas_price) = futures::join!(gas_limit_req, gas_price_req);
            let (gas_limit, v3_relay_data) = tuple?;
            let mut gas_fee = gas_limit * gas_price?;

            if let Some(price) = eth_price {
                gas_fee = Self::calculate_fee_in_token(&gas_fee, price, 6);
            }

            if has_message {
                gas_fee *= U256::from(MESSAGE_GAS_MULTIPLIER);
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
        let ctx = self.build_context(request)?;
        let pool_state = self.fetch_pool_state(&ctx).await?;

        let rate_model = Self::get_rate_model(&ctx.input_asset, &ctx.output_asset, &pool_state.token_config);
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&pool_state.utilization_before, &pool_state.utilization_after, false);
        let lpfee = fees::multiply(ctx.from_amount, lpfee_percent, ctx.capital_cost.decimals);
        let relayer_fee = Self::calculate_relayer_capital_fee(ctx.from_amount, &ctx.capital_cost);

        if lpfee + relayer_fee >= ctx.from_amount {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }
        let remain_amount = ctx.from_amount - lpfee - relayer_fee;

        let output_token_evm = if ctx.to_chain.chain_type() == ChainType::Ethereum {
            Some(eth_address::parse_asset_id(&ctx.output_asset)?)
        } else {
            None
        };

        let initial_destination_message = self.build_destination_message(&ctx, &remain_amount, output_token_evm.as_ref())?;
        let output_token_bytes = Self::token_bytes32_for_asset(&ctx.output_asset)?;
        let (gas_fee, mut v3_relay_data) = self
            .calculate_gas_price_and_fee(
                &ctx,
                &initial_destination_message,
                output_token_bytes,
                pool_state.eth_price.as_ref(),
                pool_state.sol_price.as_ref(),
            )
            .await?;

        if remain_amount <= gas_fee {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }
        let output_amount = remain_amount - gas_fee;

        let final_destination_message = self.build_destination_message(&ctx, &output_amount, output_token_evm.as_ref())?;
        let recipient_bytes = Self::recipient_to_fixed_bytes(&final_destination_message.recipient)?;
        if v3_relay_data.recipient != recipient_bytes {
            v3_relay_data.recipient = recipient_bytes;
        }
        let final_referral_fee = self.update_v3_relay_data(&mut v3_relay_data, &output_amount, pool_state.timestamp, final_destination_message);
        if final_referral_fee > output_amount {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }
        let to_value = output_amount - final_referral_fee;

        let encoded_data = v3_relay_data.abi_encode();
        let route_data = HexEncode(encoded_data);

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: ctx.input_asset.clone(),
                    output: ctx.output_asset.clone(),
                    route_data,
                    gas_limit: Some(DEFAULT_DEPOSIT_GAS_LIMIT.to_string()),
                }],
            },
            request: request.clone(),
            eta_in_seconds: self.get_eta_in_seconds(&ctx.from_chain, &ctx.to_chain),
        })
    }

    async fn fetch_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_chain = quote.request.from_asset.chain();
        if from_chain == Chain::Solana {
            if quote.data.routes.is_empty() {
                return Err(SwapperError::InvalidRoute);
            }
            let route = &quote.data.routes[0];
            let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
            let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;

            return solana_tx::build_deposit_tx(self.rpc_provider.clone(), quote, &v3_relay_data).await;
        }

        let deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let dst_chain_id = Self::get_destination_chain_id(&quote.request.to_asset.chain())?;
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let depositor = Self::decode_address_bytes32(&eth_address::parse_str(&quote.request.wallet_address)?);
        let recipient = v3_relay_data.recipient;

        let input_asset_id = quote.request.from_asset.asset_id();
        let input_token = Self::token_bytes32_for_asset(&input_asset_id)?;

        let to_asset_id = quote.request.to_asset.asset_id();
        let output_token = Self::token_bytes32_for_asset(&to_asset_id)?;

        let deposit_call = V3SpokePoolInterface::depositCall {
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

        let approval: Option<ApprovalData> = {
            if input_is_native {
                None
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    eth_address::parse_asset_id(&quote.request.from_asset.asset_id())?.to_string(),
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
            let tx = TransactionObject::new_call_to_value(&to, &hex_value, deposit_call.clone());
            let _gas_limit = self.estimate_gas_transaction(from_chain, tx).await?;
            gas_limit = Some(_gas_limit.to_string());
        }

        Ok(SwapperQuoteData::new_contract(
            deployment.spoke_pool.into(),
            value.to_string(),
            HexEncode(deposit_call.clone()),
            approval,
            gas_limit,
        ))
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let api = AcrossApi::new(self.rpc_provider.clone());
        let status = api.deposit_status(chain, transaction_hash).await?;

        let swap_status = status.swap_status();
        let destination_chain = Chain::from_chain_id(status.destination_chain_id);

        let (to_chain, to_tx_hash) = match swap_status {
            SwapStatus::Completed => (destination_chain, status.fill_tx.clone()),
            SwapStatus::Failed | SwapStatus::Refunded => (Some(chain), None),
            SwapStatus::Pending => (destination_chain, None),
        };

        Ok(SwapResult {
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
    use crate::alien::mock::{MockFn, ProviderMock};
    use crate::config::ReferralFee;
    use crate::{SwapperMode, SwapperQuoteAsset};
    use gem_evm::{
        across::contracts::{multicall_handler, spoke_pool::V3SpokePoolInterface::depositCall},
        multicall3::IMulticall3,
        weth::WETH9,
    };
    use primitives::asset_constants::*;
    use std::time::Duration;

    fn make_quote_asset(asset_id: &AssetId, decimals: u32) -> SwapperQuoteAsset {
        SwapperQuoteAsset {
            id: asset_id.to_string(),
            symbol: String::new(),
            decimals,
        }
    }

    fn make_request(from_asset: AssetId, to_asset: AssetId, wallet: &str, destination: &str, value: &str) -> QuoteRequest {
        QuoteRequest {
            from_asset: make_quote_asset(&from_asset, 18),
            to_asset: make_quote_asset(&to_asset, 18),
            wallet_address: wallet.into(),
            destination_address: destination.into(),
            value: value.into(),
            mode: SwapperMode::ExactIn,
            options: Options::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_quote_context<'a>(
        _request: &'a QuoteRequest,
        from_amount: U256,
        wallet_address: &str,
        from_chain: Chain,
        to_chain: Chain,
        input_asset: AssetId,
        output_asset: AssetId,
        original_output_asset: AssetId,
        referral_fee: ReferralFee,
        solana_destination_address: Option<&'a str>,
        input_is_native: bool,
        output_token_decimals: u8,
    ) -> QuoteContext<'a> {
        let depositor = RelayRecipient::Evm(Address::from_str(wallet_address).unwrap());

        QuoteContext {
            from_amount,
            depositor,
            evm_address: Address::from_str(wallet_address).unwrap(),
            from_chain,
            to_chain,
            input_is_native,
            input_asset,
            output_asset,
            original_output_asset,
            mainnet_token: Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
            capital_cost: fees::CapitalCostConfig {
                lower_bound: BigInt::from(0),
                upper_bound: BigInt::from(0),
                cutoff: BigInt::from(1),
                decimals: output_token_decimals as u32,
            },
            referral_fee,
            destination_deployment: AcrossDeployment::deployment_by_chain(&to_chain).unwrap(),
            solana_destination_address,
            output_token_decimals,
        }
    }

    fn mock_provider(response: &str) -> Arc<ProviderMock> {
        let response = response.to_string();
        Arc::new(ProviderMock {
            response: MockFn(Box::new(move |_| response.clone())),
            timeout: Duration::from_millis(50),
        })
    }

    #[test]
    fn test_is_supported_pair() {
        let weth_eth: AssetId = AssetId::from_token(Chain::Ethereum, WETH_ETH_CONTRACT);
        let weth_op: AssetId = AssetId::from_token(Chain::Optimism, WETH_OP_CONTRACT);
        let weth_arb: AssetId = AssetId::from_token(Chain::Arbitrum, WETH_ARB_CONTRACT);
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

        let eth = AssetId::from(Chain::Ethereum, None);
        let op = AssetId::from(Chain::Optimism, None);
        let arb = AssetId::from(Chain::Arbitrum, None);
        let linea = AssetId::from(Chain::Linea, None);

        assert!(Across::is_supported_pair(&eth, &linea));
        assert!(Across::is_supported_pair(&op, &eth));
        assert!(Across::is_supported_pair(&arb, &eth));
        assert!(Across::is_supported_pair(&op, &arb));

        let solana_usdc = SOLANA_USDC.id.clone();

        assert!(!Across::is_supported_pair(&usdc_eth, &solana_usdc));
        assert!(!Across::is_supported_pair(&usdc_arb, &solana_usdc));

        let solana_usdt = SOLANA_USDT.id.clone();

        assert!(!Across::is_supported_pair(&usdt_eth, &solana_usdt));
        assert!(!Across::is_supported_pair(&solana_usdt, &usdt_eth));
        assert!(!Across::is_supported_pair(&usdc_eth, &solana_usdt));
        assert!(!Across::is_supported_pair(&solana_usdt, &usdc_eth));

        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_eth));
        assert!(!Across::is_supported_pair(&solana_usdc, &usdc_arb));
        assert!(!Across::is_supported_pair(&weth_eth, &solana_usdc));
        assert!(!Across::is_supported_pair(&weth_eth, &solana_usdt));
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
        let depositor_addr = Address::from_str("0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7").unwrap();
        let input_token_addr = Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap();
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
    fn test_build_destination_message_eth_to_base() {
        let across = Across::new(mock_provider("{}"));
        let amount = U256::from_str("1000000000000000000").unwrap();
        let request = make_request(
            AssetId::from_chain(Chain::Ethereum),
            AssetId::from_chain(Chain::Base),
            "0x1111111111111111111111111111111111111111",
            "11111111111111111111111111111111",
            amount.to_string().as_str(),
        );
        let referral_fee = ReferralFee {
            address: "0x2222222222222222222222222222222222222222".into(),
            bps: 100,
        };
        let fee_address = Address::from_str(&referral_fee.address).unwrap();
        let ctx = make_quote_context(
            &request,
            amount,
            &request.wallet_address,
            Chain::Ethereum,
            Chain::Base,
            AssetId::from_chain(Chain::Ethereum),
            AssetId::from_token(Chain::Base, "0x4200000000000000000000000000000000000006"),
            AssetId::from_chain(Chain::Base),
            referral_fee,
            None,
            true,
            18,
        );

        let output_token = Address::from_str("0x4200000000000000000000000000000000000006").unwrap();
        let destination_message = across.build_destination_message(&ctx, &amount, Some(&output_token)).unwrap();

        let expected_fee = amount * U256::from(100u64) / U256::from(10000u64);
        assert_eq!(destination_message.referral_fee, expected_fee);

        let instructions = multicall_handler::Instructions::abi_decode(&destination_message.bytes).unwrap();
        assert_eq!(instructions.fallbackRecipient, ctx.evm_address);
        assert_eq!(instructions.calls.len(), 2);

        let expected_withdraw = WETH9::withdrawCall { wad: amount }.abi_encode();
        assert_eq!(instructions.calls[0].target, output_token);
        assert_eq!(instructions.calls[0].callData, Bytes::from(expected_withdraw));
        assert_eq!(instructions.calls[1].target, fee_address);
        assert_eq!(instructions.calls[1].value, expected_fee);
    }

    #[test]
    fn test_build_destination_message_usdc_to_optimism() {
        let across = Across::new(mock_provider("{}"));
        let amount = U256::from(1_000_000u64);
        let request = make_request(
            AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"),
            USDC_OP_ASSET_ID.into(),
            "0x1111111111111111111111111111111111111111",
            "11111111111111111111111111111111",
            amount.to_string().as_str(),
        );
        let referral_fee = ReferralFee {
            address: "0x2222222222222222222222222222222222222222".into(),
            bps: 100,
        };
        let fee_address = Address::from_str(&referral_fee.address).unwrap();
        let ctx = make_quote_context(
            &request,
            amount,
            &request.wallet_address,
            Chain::Arbitrum,
            Chain::Optimism,
            AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"),
            USDC_OP_ASSET_ID.into(),
            USDC_OP_ASSET_ID.into(),
            referral_fee,
            None,
            false,
            6,
        );

        let token_address = Address::from_str("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85").unwrap();
        let destination_message = across.build_destination_message(&ctx, &amount, Some(&token_address)).unwrap();

        let expected_fee = amount * U256::from(100u64) / U256::from(10000u64);
        assert_eq!(destination_message.referral_fee, expected_fee);

        let instructions = multicall_handler::Instructions::abi_decode(&destination_message.bytes).unwrap();
        assert_eq!(instructions.calls.len(), 1);
        assert_eq!(instructions.calls[0].target, token_address);
        assert_eq!(instructions.calls[0].value, U256::from(0));
        let fee_call = IERC20::transferCall::abi_decode(&instructions.calls[0].callData).unwrap();
        assert_eq!(fee_call.to, fee_address);
        assert_eq!(fee_call.value, expected_fee);
    }

    #[test]
    fn test_build_destination_message_solana_with_referral() {
        let across = Across::new(mock_provider("{}"));
        let amount = U256::from(2_000_000u64);
        let destination = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let referral_address = "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy";
        let request = make_request(
            AssetId::from_token(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            SOLANA_USDC.id.clone(),
            "0x1111111111111111111111111111111111111111",
            destination,
            amount.to_string().as_str(),
        );
        let referral_fee = ReferralFee {
            address: referral_address.into(),
            bps: 100,
        };
        let ctx = make_quote_context(
            &request,
            amount,
            &request.wallet_address,
            Chain::Ethereum,
            Chain::Solana,
            AssetId::from_token(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            SOLANA_USDC.id.clone(),
            SOLANA_USDC.id.clone(),
            referral_fee,
            Some(destination),
            false,
            6,
        );

        let destination_message = across.build_destination_message(&ctx, &amount, None).unwrap();
        let expected_fee = amount * U256::from(100u64) / U256::from(10000u64);
        assert_eq!(destination_message.referral_fee, expected_fee);

        let across_message: AcrossPlusMessage = borsh::from_slice(&destination_message.bytes).unwrap();
        assert_eq!(across_message.read_only_len, 2);

        let mint = SolanaPubkey::from_str(SOLANA_USDC.id.token_id.as_ref().unwrap()).unwrap();
        let user_pubkey = SolanaPubkey::from_str(destination).unwrap();
        let referral_pubkey = SolanaPubkey::from_str(referral_address).unwrap();
        let user_token_account = get_associated_token_address(&user_pubkey, &mint);
        let referral_token_account = get_associated_token_address(&referral_pubkey, &mint);

        assert!(across_message.accounts.iter().any(|acc| *acc == user_token_account));
        assert!(across_message.accounts.iter().any(|acc| *acc == referral_token_account));

        let compiled: Vec<CompiledIx> = borsh::from_slice(&across_message.handler_message).unwrap();
        assert_eq!(compiled.len(), 2);
        assert_eq!(compiled[0].account_key_indexes.len(), 3);
        assert_eq!(compiled[0].account_key_indexes, vec![0, 1, 3]);
        assert_eq!(compiled[1].account_key_indexes, vec![0, 2, 3]);
    }

    #[tokio::test]
    async fn test_relay_data_recipient_destination() {
        let across = Across::new(mock_provider("{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"0x5208\"}"));
        let amount = U256::from(12345u64);
        let wallet = "0x1111111111111111111111111111111111111111";
        let request = make_request(
            AssetId::from_token(Chain::Ethereum, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            USDC_OP_ASSET_ID.into(),
            wallet,
            wallet,
            amount.to_string().as_str(),
        );
        let input_asset = AssetId::from_token(Chain::Ethereum, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        let output_token = Across::decode_address_bytes32(&Address::from_str("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85").unwrap());
        let ctx = make_quote_context(
            &request,
            amount,
            wallet,
            Chain::Ethereum,
            Chain::Optimism,
            input_asset.clone(),
            USDC_OP_ASSET_ID.into(),
            USDC_OP_ASSET_ID.into(),
            ReferralFee::default(),
            None,
            true,
            6,
        );

        let empty_message = DestinationMessage {
            bytes: vec![],
            referral_fee: U256::ZERO,
            recipient: RelayRecipient::Evm(Address::from_str(wallet).unwrap()),
        };
        let (gas_limit, v3_relay_data) = across.estimate_gas_limit(&ctx, &empty_message, output_token).await.unwrap();

        assert_eq!(gas_limit, U256::from(21000u64));

        let expected_recipient_user = Across::decode_address_bytes32(&Address::from_str(wallet).unwrap());

        assert_eq!(v3_relay_data.recipient, expected_recipient_user);

        let expected_input_token = Across::decode_address_bytes32(&Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());

        assert_eq!(v3_relay_data.inputToken, expected_input_token);
        assert_eq!(v3_relay_data.outputToken, output_token);

        let multicall_addr = Address::from_str(ctx.destination_deployment.multicall_handler().as_str()).unwrap();
        let message = DestinationMessage {
            bytes: vec![0x01],
            referral_fee: U256::ZERO,
            recipient: RelayRecipient::Evm(multicall_addr),
        };
        let (gas_limit2, v3_relay_data2) = across.estimate_gas_limit(&ctx, &message, output_token).await.unwrap();

        assert_eq!(gas_limit2, U256::from(21000u64));

        let expected_recipient_mc = Across::decode_address_bytes32(&multicall_addr);

        assert_eq!(v3_relay_data2.recipient, expected_recipient_mc);
        assert_eq!(v3_relay_data2.inputToken, expected_input_token);
        assert_eq!(v3_relay_data2.outputToken, output_token);

        let base_weth = "0x4200000000000000000000000000000000000006";
        let output_token_base = Across::decode_address_bytes32(&Address::from_str(base_weth).unwrap());
        let base_ctx = make_quote_context(
            &request,
            amount,
            wallet,
            Chain::Ethereum,
            Chain::Base,
            input_asset.clone(),
            AssetId::from_token(Chain::Base, base_weth),
            AssetId::from_chain(Chain::Base),
            ReferralFee::default(),
            None,
            true,
            18,
        );
        let base_message = DestinationMessage {
            bytes: vec![],
            referral_fee: U256::ZERO,
            recipient: RelayRecipient::Evm(Address::from_str(wallet).unwrap()),
        };
        let (gas_limit3, v3_relay_data3) = across.estimate_gas_limit(&base_ctx, &base_message, output_token_base).await.unwrap();

        assert_eq!(gas_limit3, U256::from(21000u64));

        let expected_input_token_eth_weth = Across::decode_address_bytes32(&Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());
        let expected_output_token_base_weth = Across::decode_address_bytes32(&Address::from_str(base_weth).unwrap());

        assert_eq!(v3_relay_data3.inputToken, expected_input_token_eth_weth);
        assert_eq!(v3_relay_data3.outputToken, expected_output_token_base_weth);
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
                value: "20000000000000000".into(),
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
                value: "50000000".into(),
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
        async fn test_across_quote_eth_usdc_to_solana_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                fee: None,
                preferred_providers: vec![],
                use_max_amount: false,
            };

            let wallet = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let destination = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
            let from_asset: AssetId = USDC_ETH_ASSET_ID.into();
            let to_asset: AssetId = USDC_SOLANA_ASSET_ID.into();
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: wallet.into(),
                destination_address: destination.into(),
                value: "1000000".into(),
                mode: SwapperMode::ExactIn,
                options,
            };

            let result = swap_provider.fetch_quote(&request).await;
            match result {
                Err(err) => assert_eq!(err, SwapperError::NoQuoteAvailable),
                Ok(_) => panic!("expected NoQuoteAvailable"),
            }

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_solana_usdc_to_eth_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                fee: None,
                preferred_providers: vec![],
                use_max_amount: false,
            };

            let wallet = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
            let destination = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let from_asset: AssetId = USDC_SOLANA_ASSET_ID.into();
            let to_asset: AssetId = USDC_ETH_ASSET_ID.into();
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: wallet.into(),
                destination_address: destination.into(),
                value: "1000000".into(),
                mode: SwapperMode::ExactIn,
                options,
            };

            let result = swap_provider.fetch_quote(&request).await;
            match result {
                Err(err) => assert_eq!(err, SwapperError::NoQuoteAvailable),
                Ok(_) => panic!("expected NoQuoteAvailable"),
            }

            Ok(())
        }

        #[tokio::test]
        async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::new(network_provider.clone());

            let tx_hash = "0x9827ca4bdd5dea3a310cff3485f87463987cdc52118077dba34f86ee79456952";
            let chain = Chain::Unichain;

            let result = swap_provider.get_swap_result(chain, tx_hash).await?;

            println!("Across swap result: {:?}", result);
            assert_eq!(result.from_chain, chain);
            assert_eq!(result.from_tx_hash, tx_hash);
            assert_eq!(result.status, SwapStatus::Completed);
            assert_eq!(result.to_chain, Some(Chain::Linea));
            assert_eq!(result.to_tx_hash, Some("0xcba653515ab00f5b3ebc16eb4d099e29611e1e59b3fd8f2800cf2302d175f9fe".to_string()));

            Ok(())
        }
    }
}
