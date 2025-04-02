use std::sync::Arc;

mod client;
mod model;

use crate::{
    config::swap_config::SwapReferralFee,
    network::AlienProvider,
    swapper::{asset::*, SwapChainAsset},
};
use async_trait::async_trait;
use client::ProxyClient;
use model::{Quote, QuoteRequest};
use primitives::{Chain, ChainType};

use super::{
    FetchQuoteData, GemSwapOptions, GemSwapProvider, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute,
    SwapperError,
};

const PROVIDER_URL: &str = "https://api.gemwallet.com/swapper";

#[derive(Debug)]
pub struct ProxyProvider {
    pub provider: SwapProviderType,
    pub url: String,
}

impl ProxyProvider {
    pub fn new_stonfi_v2() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::StonFiV2),
            url: format!("{}/{}", PROVIDER_URL, "stonfi_v2"),
        }
    }

    pub fn new_mayan() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::Mayan),
            url: format!("{}/{}", PROVIDER_URL, "mayan"),
        }
    }

    fn get_referrer(&self, chain: &Chain, options: &GemSwapOptions, provider: &SwapProvider) -> SwapReferralFee {
        match provider {
            // always use solana for Mayan, otherwise not supported chain error
            SwapProvider::Mayan => {
                return options.fee.as_ref().unwrap().solana.clone();
            }
            _ => {}
        }

        match chain.chain_type() {
            ChainType::Ethereum => options.fee.as_ref().unwrap().evm.clone(),
            ChainType::Solana => options.fee.as_ref().unwrap().solana.clone(),
            ChainType::Ton => options.fee.as_ref().unwrap().ton.clone(),
            ChainType::Sui => options.fee.as_ref().unwrap().sui.clone(),
            _ => SwapReferralFee::default(),
        }
    }
}

#[async_trait]
impl GemSwapProvider for ProxyProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            // StoneFiV2
            SwapChainAsset::All(Chain::Ton),
            // Mayan
            SwapChainAsset::Assets(
                Chain::Ethereum,
                vec![
                    ETHEREUM_USDT.id.clone(),
                    ETHEREUM_USDC.id.clone(),
                    ETHEREUM_DAI.id.clone(),
                    ETHEREUM_USDS.id.clone(),
                    ETHEREUM_BNB.id.clone(),
                    ETHEREUM_FDUSD.id.clone(),
                    ETHEREUM_WBTC.id.clone(),
                    ETHEREUM_WETH.id.clone(),
                    ETHEREUM_STETH.id.clone(),
                    ETHEREUM_CBBTC.id.clone(),
                ],
            ),
            SwapChainAsset::Assets(
                Chain::Solana,
                vec![
                    SOLANA_USDC.id.clone(),
                    SOLANA_USDT.id.clone(),
                    SOLANA_USDS.id.clone(),
                    SOLANA_CBBTC.id.clone(),
                    SOLANA_WBTC.id.clone(),
                    SOLANA_JITO_SOL.id.clone(),
                ],
            ),
            SwapChainAsset::Assets(
                Chain::SmartChain,
                vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone(), SMARTCHAIN_WBTC.id.clone()],
            ),
            SwapChainAsset::Assets(
                Chain::Base,
                vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone(), BASE_WBTC.id.clone(), BASE_USDS.id.clone()],
            ),
            SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone(), POLYGON_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let client = ProxyClient::new(provider);
        let referrer = self.get_referrer(&request.from_asset.chain, &request.options, &self.provider.id);
        let quote_request = QuoteRequest {
            from_address: request.wallet_address.clone(),
            from_asset: request.from_asset.to_string(),
            to_address: request.destination_address.clone(),
            to_asset: request.to_asset.to_string(),
            from_value: request.value.clone(),
            referral_address: referrer.address.clone(),
            referral_bps: referrer.bps as usize,
            slippage_bps: request.options.slippage.bps as usize,
        };

        let quote = client.get_quote(&self.url, quote_request.clone()).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote.output_value.clone(),
            to_min_value: quote.output_value.clone(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: serde_json::to_string(&quote).unwrap(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let routes = quote.data.clone().routes;
        let route_data: Quote = serde_json::from_str(&routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let client = ProxyClient::new(provider);
        let data = client.get_quote_data(&self.url, route_data).await?;

        Ok(SwapQuoteData {
            to: data.to,
            value: data.value,
            data: data.data,
            approval: None,
            gas_limit: None,
        })
    }
}
