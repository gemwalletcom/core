use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use gem_hypercore::core::{HYPE_SYSTEM_ADDRESS, HYPERCORE_HYPE_TOKEN, actions::user::spot_send::SpotSend, hypercore::transfer_to_hyper_evm_typed_data};
use number_formatter::BigNumberFormatter;

use primitives::{
    Chain,
    swap::{SwapResult, SwapStatus},
};

use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    asset::{HYPERCORE_HYPE, HYPEREVM_HYPE},
};

use super::math::scale_quote_value;

#[derive(Debug)]
pub struct HyperCoreBridge {
    provider: ProviderType,
}

impl HyperCoreBridge {
    pub fn new() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperliquid),
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
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_HYPE.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_HYPE.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let to_value = scale_quote_value(&request.value, request.from_asset.decimals, request.to_asset.decimals)?;

        let quote = Quote {
            from_value: request.value.clone(),
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: 0,
                routes: vec![Route {
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

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        match quote.request.from_asset.asset_id().chain {
            Chain::HyperCore => {
                let decimals: i32 = quote.request.from_asset.decimals.try_into().unwrap();
                let amount = BigNumberFormatter::value(&quote.request.value, decimals)?;
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;

                let spot_send = SpotSend::new(amount, HYPE_SYSTEM_ADDRESS.to_string(), timestamp, HYPERCORE_HYPE_TOKEN.to_string());
                let typed_data = transfer_to_hyper_evm_typed_data(spot_send);

                Ok(SwapperQuoteData::new_contract(
                    HYPE_SYSTEM_ADDRESS.to_string(),
                    quote.request.value.clone(),
                    typed_data,
                    None,
                    None,
                ))
            }
            Chain::Hyperliquid => Ok(SwapperQuoteData::new_contract(
                HYPE_SYSTEM_ADDRESS.to_string(),
                quote.request.value.clone(),
                "0x".to_string(),
                None,
                None,
            )),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let from_chain = chain;
        let to_chain = if chain == Chain::HyperCore { Chain::Hyperliquid } else { Chain::HyperCore };
        Ok(SwapResult {
            status: SwapStatus::Completed,
            from_chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain: Some(to_chain),
            to_tx_hash: None,
        })
    }
}
