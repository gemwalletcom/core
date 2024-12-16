// FIXME remove this
#![allow(unused_variables)]
#![allow(unused_imports)]

use super::{
    api::AcrossApi,
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
};
use crate::{
    network::AlienProvider,
    swapper::{
        approval::{self, check_approval, CheckApprovalType},
        models::*,
        slippage::apply_slippage_in_bp,
        weth_address, GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    across::{
        deployment::{AcrossDeployment, ACROSS_CONFIG_STORE, ACROSS_HUBPOOL},
        fees::{LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    address::EthereumAddress,
};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U256,
    },
    sol_types::{abi::token, SolCall},
};
use async_trait::async_trait;
use std::{f32::consts::E, fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug, Default)]
pub struct Across {}

impl Across {
    pub fn boxed() -> Box<dyn GemSwapProvider> {
        Box::new(Self::default())
    }

    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        let from = weth_address::normalize_asset(from_asset).unwrap();
        let to = weth_address::normalize_asset(to_asset).unwrap();

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

    pub async fn is_paused(&self, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let hub_client = HubPoolClient {
            contract: ACROSS_HUBPOOL.into(),
            provider: provider.clone(),
            chain: Chain::Ethereum,
        };
        hub_client.is_paused().await
    }
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_chains(&self) -> Vec<Chain> {
        AcrossDeployment::deployed_chains()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // does not support same chain swap
        if request.from_asset.chain == request.to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }

        let from_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let from_amount: U256 = request.value.parse().map_err(|_| SwapperError::InvalidAmount)?;
        let deployment = AcrossDeployment::deployment_by_chain(&request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset, &request.to_asset) {
            return Err(SwapperError::NotSupportedPair);
        }

        let is_paused = self.is_paused(provider.clone()).await?;
        if is_paused {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_asset = weth_address::normalize_asset(&request.from_asset).unwrap();
        let output_asset = weth_address::normalize_asset(&request.to_asset).unwrap();

        let mappings = AcrossDeployment::asset_mappings();
        let asset_mapping = mappings.iter().find(|x| x.set.contains(&input_asset)).unwrap();
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).unwrap();
        let mainnet_token = weth_address::parse_into_address(asset_mainnet, from_chain)?;

        let config_client = ConfigStoreClient {
            contract: ACROSS_CONFIG_STORE.into(),
            provider: provider.clone(),
            chain: Chain::Ethereum,
        };
        let hubpool_client = HubPoolClient {
            contract: ACROSS_HUBPOOL.into(),
            provider: provider.clone(),
            chain: Chain::Ethereum,
        };

        // FIXME: batch call
        let token_config = config_client.fetch_config(&mainnet_token).await?;
        let util_before = hubpool_client.fetch_utilization(&mainnet_token, U256::from(0)).await?;
        let util_after = hubpool_client.fetch_utilization(&mainnet_token, from_amount).await?;

        let rate_model = Self::get_rate_model(&request.from_asset, &request.to_asset, &token_config);
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);

        let cost_config = &asset_mapping.capital_cost;
        let relayer_fee_percent = RelayerFeeCalculator::capital_fee_percent(BigInt::from_str(&request.value).expect("valid amount"), cost_config);

        // FIXME: calculate referrer fee
        // FIXME: build message

        // FIXME: prepare fill tx to estimate gas limit
        // FIXME: get gas price from destination chain

        let approval = ApprovalType::None;

        // FIXME: check Token approval
        // FIXME: calculate gas fee amount in token amount

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: request.value.clone(),
            data: SwapProviderData {
                provider: self.provider(),
                suggested_slippage_bps: None,
                routes: vec![SwapRoute {
                    input: input_asset.clone(),
                    output: output_asset.clone(),
                    route_data: "".into(),
                    gas_estimate: None,
                }],
            },
            approval,
            request: request.clone(),
        })
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        Err(SwapperError::NotImplemented)
    }
    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let api = AcrossApi::new(provider.clone());
        let status = api.deposit_status(chain.network_id(), transaction_hash).await?;
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
    }
}
