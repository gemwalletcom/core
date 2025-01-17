use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::{hex, Address};
use async_trait::async_trait;
use gem_evm::stargate::contract::{MessagingFee, SendParam};
use primitives::{Chain, CryptoValueConverter};
use serde::{Deserialize, Serialize};

use crate::{
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20, slippage::apply_slippage_in_bp, ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider,
        SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{client::StargateClient, endpoint::STARGATE_ROUTES, layer_zero::scan::LayerZeroScanApi};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StargateRouteData {
    send_param: SendParam,
    fee: MessagingFee,
    refund_address: String,
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

        if from_asset.chain == to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }

        if from_asset.is_native() && !to_asset.is_native() {
            return Err(SwapperError::NotSupportedPair);
        }

        let pool = self.client.get_pool_by_asset_id(&request.from_asset)?;
        let mut send_param = self.client.build_send_param(request)?;

        let oft_quote = self.client.quote_oft(pool, &send_param, provider.clone()).await?;
        send_param.minAmountLD = apply_slippage_in_bp(&oft_quote.receipt.amountReceivedLD, request.options.slippage_bps);
        let messaging_fee = self.client.quote_send(pool, &send_param, provider.clone()).await?;

        let approval = if request.from_asset.is_token() {
            check_approval_erc20(
                request.wallet_address.clone(),
                request.from_asset.token_id.clone().unwrap(),
                pool.address.clone(),
                send_param.amountLD,
                provider.clone(),
                &request.from_asset.chain,
            )
            .await?
        } else {
            ApprovalType::None
        };

        let route_data = StargateRouteData {
            send_param: send_param.clone(),
            fee: messaging_fee,
            refund_address: request.wallet_address.to_string(),
        };

        let from_decimals = self.client.get_decimals_by_asset_id(&request.from_asset)?;
        let to_decimals = self.client.get_decimals_by_asset_id(&request.to_asset)?;
        let mut to_value = CryptoValueConverter::value_from(oft_quote.receipt.amountReceivedLD.to_string(), from_decimals);
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
                    gas_estimate: None,
                }],
                suggested_slippage_bps: None,
            },
            approval,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let pool = self.client.get_pool_by_asset_id(&quote.request.from_asset)?;
        let route_data: StargateRouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let send_calldata = self.client.send(
            &route_data.send_param,
            &route_data.fee,
            &Address::from_str(route_data.refund_address.as_str()).unwrap(),
        );

        let mut value_to_send = route_data.fee.nativeFee;

        if quote.request.from_asset.is_native() {
            value_to_send += route_data.send_param.amountLD;
        }

        let quote_data = SwapQuoteData {
            to: pool.address.clone(),
            value: value_to_send.to_string(),
            data: hex::encode_prefixed(send_calldata.clone()),
        };

        Ok(quote_data)
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let api = LayerZeroScanApi::new(provider.clone());
        let response = api.get_message_by_tx(transaction_hash).await?;
        let messages = response.data;
        let message = messages.first().ok_or(SwapperError::NetworkError {
            msg: "Unable to check transaction status using Stargate Provider: No message found".into(),
        })?;
        Ok(message.status.is_delivered())
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
    async fn test_same_chain_quote() {
        let stargate = Stargate::new();
        let request = SwapQuoteRequest {
            wallet_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            from_asset: AssetId::from_chain(Chain::Ethereum),
            to_asset: AssetId::from_chain(Chain::Ethereum),
            value: U256::from(1).to_string(),
            mode: GemSwapMode::ExactIn,
            destination_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            options: GemSwapOptions {
                slippage_bps: 100,
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
                slippage_bps: 100,
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
