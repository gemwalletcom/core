use super::{
    api::AcrossApi,
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
};
use crate::{
    debug_println,
    network::AlienProvider,
    swapper::{approval::check_approval_erc20, asset::*, eth_rpc, models::*, slippage::apply_slippage_in_bp, weth_address, GemSwapProvider, SwapperError},
};
use gem_evm::{
    across::{
        contracts::V3SpokePoolInterface::{self, V3RelayData},
        deployment::{AcrossDeployment, ACROSS_CONFIG_STORE, ACROSS_HUBPOOL},
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    address::EthereumAddress,
    jsonrpc::TransactionObject,
};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    hex::decode as HexDecode,
    hex::encode_prefixed as HexEncode,
    primitives::{Address, Bytes, U256},
    sol_types::{SolCall, SolValue},
};
use async_trait::async_trait;
use std::{
    fmt::Debug,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Default)]
pub struct Across {}

impl Across {
    pub fn boxed() -> Box<dyn GemSwapProvider> {
        Box::new(Self::default())
    }

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        let from = weth_address::normalize_asset(from_asset).unwrap();
        let to = weth_address::normalize_asset(to_asset).unwrap();
        debug_println!("from: {:?}, to: {:?}", from, to);
        let asset_mappings = AcrossDeployment::asset_mappings();
        for mapping in asset_mappings.iter() {
            if mapping.set.contains(&from) && mapping.set.contains(&to) {
                return true;
            }
        }
        false
    }

    pub fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    pub async fn estimate_gas_limit(
        &self,
        amount: &U256,
        is_native: bool,
        input_asset: &AssetId,
        output_asset: &AssetId,
        wallet_address: &EthereumAddress,
        deployment: &AcrossDeployment,
        provider: Arc<dyn AlienProvider>,
        chain: &Chain,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain_id: u32 = chain.network_id().parse().unwrap();

        let v3_relay_data = V3RelayData {
            depositor: Address::from_slice(&wallet_address.bytes),
            recipient: Address::from_slice(&wallet_address.bytes),
            exclusiveRelayer: Address::ZERO,
            inputToken: Address::from_str(input_asset.token_id.clone().unwrap().as_ref()).unwrap(),
            outputToken: Address::from_str(output_asset.token_id.clone().unwrap().as_ref()).unwrap(),
            inputAmount: *amount,
            outputAmount: U256::from(100), // safe amount
            originChainId: U256::from(chain_id),
            depositId: u32::MAX,
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(vec![]),
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
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        // WETH for now
        vec![
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Base, vec![BASE_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Blast, vec![BLAST_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Linea, vec![LINEA_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::ZkSync, vec![ZKSYNC_WETH.id.clone()]),
            SwapChainAsset::Assets(Chain::World, vec![WORLD_WETH.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // does not support same chain swap
        if request.from_asset.chain == request.to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }

        let from_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(|_| SwapperError::InvalidAmount)?;
        let wallet_address = EthereumAddress::parse(&request.wallet_address).ok_or(SwapperError::InvalidAddress {
            address: request.wallet_address.clone(),
        })?;

        let deployment = AcrossDeployment::deployment_by_chain(&request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset, &request.to_asset) {
            return Err(SwapperError::NotSupportedPair);
        }

        let input_asset = weth_address::normalize_asset(&request.from_asset).unwrap();
        let output_asset = weth_address::normalize_asset(&request.to_asset).unwrap();

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

        let calls = vec![
            config_client.config_call3(&mainnet_token),
            hubpool_client.utilization_call3(&mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&mainnet_token, from_amount),
        ];
        let results = eth_rpc::multicall3_call(provider.clone(), &hubpool_client.chain, calls).await?;
        let token_config = config_client.decoded_config_call3(&results[0])?;
        let util_before = hubpool_client.decoded_utilization_call3(&results[1])?;
        let util_after = hubpool_client.decoded_utilization_call3(&results[2])?;

        let rate_model = Self::get_rate_model(&request.from_asset, &request.to_asset, &token_config);
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent);

        let cost_config = &asset_mapping.capital_cost;
        let relayer_fee_percent = RelayerFeeCalculator::capital_fee_percent(BigInt::from_str(&request.value).expect("valid amount"), cost_config);
        let relayer_fee = fees::multiply(from_amount, relayer_fee_percent);

        // let referral_fee_bps = request.options.fee.clone().unwrap_or_default().evm.bps / 2;
        // let referral_fee = from_amount * U256::from(referral_fee_bps) / U256::from(10000);

        // FIXME: add referral fee and override relay data message
        let referral_fee = U256::from(0);

        let (gas_limit, mut v3_relay_data) = self
            .estimate_gas_limit(
                &from_amount,
                request.from_asset.is_native(),
                &input_asset,
                &output_asset,
                &wallet_address,
                &deployment,
                provider.clone(),
                &request.to_asset.chain,
            )
            .await?;
        let gas_price = eth_rpc::fetch_gas_price(provider.clone(), &request.to_asset.chain).await?;
        let gas_fee = gas_limit * gas_price;

        let remain_amount = from_amount - lpfee - relayer_fee - referral_fee;
        if remain_amount < gas_fee {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Bridge amount is too small".into(),
            });
        }

        // Check output amount against slippage
        let output_amount = from_amount - lpfee - relayer_fee - referral_fee - gas_fee;
        let expect_min = apply_slippage_in_bp(&from_amount, request.options.slippage_bps);
        if output_amount < expect_min {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Expected amount exceeds slippage".into(),
            });
        }

        let approval: ApprovalType = {
            if request.from_asset.is_native() {
                ApprovalType::None
            } else {
                check_approval_erc20(
                    request.wallet_address.clone(),
                    input_asset.token_id.clone().unwrap(),
                    deployment.spoke_pool.into(),
                    from_amount,
                    provider.clone(),
                    &request.from_asset.chain,
                )
                .await?
            }
        };

        v3_relay_data.outputAmount = output_amount;
        let route_data = HexEncode(v3_relay_data.abi_encode());

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: output_amount.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                suggested_slippage_bps: None,
                routes: vec![SwapRoute {
                    input: input_asset.clone(),
                    output: output_asset.clone(),
                    route_data,
                    gas_estimate: None,
                }],
            },
            approval,
            request: request.clone(),
        })
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let deployment = AcrossDeployment::deployment_by_chain(&quote.request.to_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let dst_chain_id: u32 = quote.request.to_asset.chain.network_id().parse().unwrap();
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data, true).map_err(|_| SwapperError::InvalidRoute)?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f32();

        let deposit_v3_call = V3SpokePoolInterface::depositV3Call {
            depositor: v3_relay_data.depositor,
            recipient: v3_relay_data.recipient,
            inputToken: v3_relay_data.inputToken,
            outputToken: v3_relay_data.outputToken,
            inputAmount: v3_relay_data.inputAmount,
            outputAmount: v3_relay_data.outputAmount,
            destinationChainId: U256::from(dst_chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: timestamp as u32,
            fillDeadline: v3_relay_data.exclusivityDeadline,
            exclusivityDeadline: 0,
            message: v3_relay_data.message,
        }
        .abi_encode();

        let value: &str = if quote.request.from_asset.is_native() { &quote.from_value } else { "0" };

        let quote_data = SwapQuoteData {
            to: deployment.spoke_pool.into(),
            value: value.to_string(),
            data: HexEncode(deposit_v3_call),
        };
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
    use gem_evm::constants::*;

    #[test]
    fn test_is_supported_pair() {
        let weth_eth: AssetId = WETH_ETH.into();
        let weth_op: AssetId = WETH_OP.into();
        let weth_arb: AssetId = WETH_ARB.into();

        let usdc_eth: AssetId = USDC_ETH.into();
        let usdc_arb: AssetId = USDC_ARB.into();

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
}
