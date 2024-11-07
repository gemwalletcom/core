mod client;
mod model;

use model::ThorChainAsset;
use num_bigint::BigInt;
use std::str::FromStr;

use crate::network::AlienProvider;
use crate::swapper::models::{ApprovalType, FetchQuoteData, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapperError};
use crate::swapper::thorchain::client::ThorChainSwapClient;
use crate::swapper::GemSwapProvider;
use async_trait::async_trait;
use primitives::{Asset, Chain, ChainType};
use std::sync::Arc;

use super::SwapRoute;

#[derive(Debug)]
pub struct ThorChain {}

impl ThorChain {
    pub fn new() -> Self {
        Self {}
    }

    fn data(&self, chain: Chain, memo: String) -> String {
        match chain {
            Chain::Thorchain | Chain::Litecoin | Chain::Doge | Chain::Bitcoin => memo,
            _ => hex::encode(memo.as_bytes()),
        }
    }
}

#[async_trait]
impl GemSwapProvider for ThorChain {
    fn name(&self) -> &'static str {
        "THORChain"
    }

    async fn supported_chains(&self) -> Result<Vec<Chain>, SwapperError> {
        let chains: Vec<Chain> = Chain::all()
            .into_iter()
            .filter_map(|chain| ThorChainAsset::from_chain(&chain).map(|name| name.chain()))
            .collect();
        Ok(chains)
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let endpoint = provider
            .get_endpoint(Chain::Thorchain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        let client = ThorChainSwapClient::new(provider);

        let from_decimals = Asset::from_chain(request.clone().from_asset.chain).decimals as u32;
        let to_decimals = Asset::from_chain(request.clone().to_asset.chain).decimals as u32;

        let value = BigInt::from_str(request.clone().value.as_str()).unwrap() / BigInt::from(10).pow(from_decimals - 8);

        let quote = client
            .get_quote(
                endpoint.as_str(),
                request.clone().from_asset,
                request.to_asset.clone(),
                value.to_string(),
                "g1".to_string(),
                50,
            )
            .await?;

        let to_value = BigInt::from_str(quote.expected_amount_out.as_str()).unwrap() * BigInt::from(10).pow(to_decimals - 8);

        let quote = SwapQuote {
            chain_type: ChainType::Ethereum,
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            provider: SwapProviderData {
                name: self.name().to_string(),
                routes: vec![SwapRoute {
                    route_type: quote.inbound_address.unwrap_or_default(),
                    input: request.clone().from_asset.to_string(),
                    output: request.clone().to_asset.to_string(),
                    fee_tier: "".to_string(),
                    gas_estimate: None,
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let memo = ThorChainSwapClient::get_memo(quote.request.to_asset.clone(), quote.request.destination_address.clone(), "g1".to_string(), 50).unwrap();
        let to = quote.provider.routes.first().unwrap().route_type.clone();
        let data: String = self.data(quote.request.from_asset.clone().chain, memo);

        let data = SwapQuoteData {
            to,
            value: quote.request.value.clone(),
            data,
        };

        Ok(data)
    }
}
