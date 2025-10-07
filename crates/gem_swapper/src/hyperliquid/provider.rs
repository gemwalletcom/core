use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use gem_hypercore::core::{HYPE_SYSTEM_ADDRESS, HYPERCORE_HYPE_TOKEN, actions::user::spot_send::SpotSend, hypercore::transfer_to_hyper_evm_typed_data};
use number_formatter::BigNumberFormatter;

use primitives::Chain;

use crate::{
    FetchQuoteData, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData,
    SwapperQuoteRequest, SwapperRoute, asset::HYPERCORE_HYPE,
};

#[derive(Debug)]
pub struct HyperCoreBridge {
    provider: SwapperProviderType,
}

impl HyperCoreBridge {
    pub fn new() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Hyperliquid),
        }
    }
}

impl Default for HyperCoreBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Swapper for HyperCoreBridge {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_HYPE.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPERCORE_HYPE.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        let quote = SwapperQuote {
            from_value: request.value.clone(),
            to_value: request.value.clone(),
            data: SwapperProviderData {
                provider: self.provider.clone(),
                slippage_bps: 0,
                routes: vec![SwapperRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: "".to_string(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
            eta_in_seconds: None,
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        match quote.request.from_asset.asset_id().chain {
            Chain::HyperCore => {
                let decimals: i32 = quote.request.from_asset.decimals.try_into().unwrap();
                let amount =
                    BigNumberFormatter::value(&quote.request.value, decimals).ok_or(SwapperError::TransactionError("Parsing amount error".to_string()))?;
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;

                let spot_send = SpotSend::new(amount, HYPE_SYSTEM_ADDRESS.to_string(), timestamp, HYPERCORE_HYPE_TOKEN.to_string());
                let typed_data = transfer_to_hyper_evm_typed_data(spot_send);

                Ok(SwapperQuoteData {
                    to: HYPE_SYSTEM_ADDRESS.to_string(),
                    value: quote.request.value.clone(),
                    data: typed_data,
                    approval: None,
                    gas_limit: None,
                })
            }
            Chain::Hyperliquid => Ok(SwapperQuoteData {
                to: HYPE_SYSTEM_ADDRESS.to_string(),
                value: quote.request.value.clone(),
                data: "0x".to_string(),
                approval: None,
                gas_limit: None,
            }),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }
}
