use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::{hex, Address, U256};
use async_trait::async_trait;
use gem_evm::{
    jsonrpc::TransactionObject,
    stargate::contract::{MessagingFee, SendParam},
};
use primitives::{Chain, CryptoValueConverter};
use serde::{Deserialize, Serialize};

use crate::{
    debug_println,
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20, eth_rpc, slippage::apply_slippage_in_bp, ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider,
        SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{client::StargateClient, endpoint::STARGATE_ROUTES};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StargateRouteData {
    send_param: SendParam,
    fee: MessagingFee,
    refund_address: String,
    min_value_out: U256,
}

#[derive(Debug, Default)]
pub struct Stargate {
    client: StargateClient,
}

impl Stargate {
    pub fn new() -> Self {
        let endpoints = vec![
            STARGATE_ROUTES.ethereum.clone(),
            STARGATE_ROUTES.base.clone(),
            STARGATE_ROUTES.optimism.clone(),
            STARGATE_ROUTES.arbitrum.clone(),
            STARGATE_ROUTES.polygon.clone(),
            STARGATE_ROUTES.avalanche.clone(),
            STARGATE_ROUTES.linea.clone(),
            STARGATE_ROUTES.smartchain.clone(),
        ];

        let client = StargateClient::from_endpoints(endpoints);

        Self { client }
    }
}

#[async_trait]
impl GemSwapProvider for Stargate {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Stargate
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        self.client
            .get_endpoints()
            .iter()
            .map(|x| SwapChainAsset::Assets(x.id, x.pools.iter().map(|y| y.asset.id.clone()).collect()))
            .collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let from_asset = &request.from_asset;
        let to_asset = &request.to_asset;

        if from_asset.is_native() && !to_asset.is_native() {
            return Err(SwapperError::NotSupportedPair);
        }

        let slippage_bps = request.options.slippage.bps;
        let pool = self.client.get_pool_by_asset_id(&request.from_asset)?;
        let initial_send_param = self.client.build_send_param(request)?;

        let oft_quote = self.client.quote_oft(pool, &initial_send_param, provider.clone()).await?;
        let min_amount_ld = apply_slippage_in_bp(&oft_quote.receipt.amountReceivedLD, slippage_bps);

        // We need to recalculate instructions based on receipt from OFT quote
        // Without this, the first call, where we send bridge token to recipient, will fail
        let compose_msg_with_min_amount = self.client.build_compose_msg(min_amount_ld, request)?;
        let final_send_param = SendParam {
            amountLD: initial_send_param.amountLD,
            minAmountLD: min_amount_ld,
            composeMsg: compose_msg_with_min_amount.into(),
            ..initial_send_param
        };
        let messaging_fee = self.client.quote_send(pool, &final_send_param, provider.clone()).await?;

        let approval = if request.from_asset.is_token() {
            check_approval_erc20(
                request.wallet_address.clone(),
                request.from_asset.token_id.clone().unwrap(),
                pool.address.clone(), // TODO: use different address
                final_send_param.amountLD,
                provider.clone(),
                &request.from_asset.chain,
            )
            .await?
        } else {
            ApprovalType::None
        };

        let amount = if request.from_asset.is_native() {
            final_send_param.amountLD
        } else {
            U256::ZERO
        };
        let min_value_out = messaging_fee.nativeFee + amount;

        let route_data = StargateRouteData {
            send_param: final_send_param.clone(),
            fee: messaging_fee.clone(),
            refund_address: request.wallet_address.to_string(),
            min_value_out,
        };

        let from_decimals = self.client.get_decimals_by_asset_id(&request.from_asset)?;
        let to_decimals = self.client.get_decimals_by_asset_id(&request.to_asset)?;
        let mut to_value = CryptoValueConverter::value_from(final_send_param.minAmountLD.to_string(), from_decimals);
        to_value = CryptoValueConverter::value_to(to_value.to_string(), to_decimals);

        Ok(SwapQuote {
            from_value: request.value.to_string(),
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: serde_json::to_string(&route_data).unwrap_or_default(),
                    gas: None,
                }],
                slippage_bps,
            },
            approval,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let pool = self.client.get_pool_by_asset_id(&quote.request.from_asset)?;
        let route_data: StargateRouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let calldata = self.client.send(
            &route_data.send_param,
            &route_data.fee,
            &Address::from_str(route_data.refund_address.as_str()).unwrap(),
        );

        let quote_data = SwapQuoteData {
            to: pool.address.clone(),
            value: route_data.min_value_out.to_string(),
            data: hex::encode_prefixed(calldata.clone()),
        };

        if matches!(_data, FetchQuoteData::EstimateGas) {
            let hex_value = format!("{:#x}", route_data.min_value_out);
            let tx = TransactionObject::new_call_to_value(&quote_data.to, &hex_value, calldata.clone());
            let gas_limit = eth_rpc::estimate_gas(_provider, &quote.request.from_asset.chain, tx).await?;
            debug_println!("gas_limit: {:?}", gas_limit);
        }

        Ok(quote_data)
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        // TODO: implement onchain scanner solution
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy_primitives::U256;
    use primitives::AssetId;

    use crate::{
        config::swap_config::SwapReferralFees,
        network::mock::AlienProviderMock,
        swapper::{asset::BASE_USDC, GemSwapMode, GemSwapOptions},
    };

    use super::*;

    #[test]
    fn should_contain_all_endpoints() {
        let stargate = Stargate::new();
        assert_eq!(
            stargate.client.get_endpoints(),
            vec![
                &STARGATE_ROUTES.ethereum,
                &STARGATE_ROUTES.base,
                &STARGATE_ROUTES.optimism,
                &STARGATE_ROUTES.arbitrum,
                &STARGATE_ROUTES.polygon,
                &STARGATE_ROUTES.avalanche,
                &STARGATE_ROUTES.linea,
                &STARGATE_ROUTES.smartchain,
            ]
        );
    }

    #[tokio::test]
    async fn test_native_to_erc20_quote() {
        let stargate = Stargate::new();
        let request = SwapQuoteRequest {
            wallet_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            from_asset: AssetId::from_chain(Chain::Ethereum),
            to_asset: BASE_USDC.id.clone(),
            value: U256::from(1).to_string(),
            mode: GemSwapMode::ExactIn,
            destination_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            options: GemSwapOptions {
                slippage: 100.into(),
                fee: Some(SwapReferralFees::default()),
                preferred_providers: vec![],
            },
        };

        let mock = AlienProviderMock {
            response: String::from("Hello"),
            timeout: Duration::from_millis(100),
        };

        let result = stargate.fetch_quote(&request, Arc::new(mock)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SwapperError::NotSupportedPair);
    }
}
