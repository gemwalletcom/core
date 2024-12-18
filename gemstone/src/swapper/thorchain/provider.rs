use crate::network::AlienProvider;
use crate::swapper::asset::{AVALANCHE_USDC, AVALANCHE_USDT, ETHEREUM_DAI, ETHEREUM_USDC, ETHEREUM_USDT, ETHEREUM_WBTC, SMARTCHAIN_USDC, SMARTCHAIN_USDT};
use crate::swapper::thorchain::client::ThorChainSwapClient;
use crate::swapper::{GemSwapProvider, SwapChainAsset};
use async_trait::async_trait;
use primitives::Chain;
use std::sync::Arc;

use super::{asset::THORChainAsset, chain::THORChainName, ThorChain, QUOTE_INTERVAL, QUOTE_MINIMUM, QUOTE_QUANTITY};
use crate::swapper::{ApprovalType, FetchQuoteData, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError};

#[async_trait]
impl GemSwapProvider for ThorChain {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Thorchain
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all()
            .into_iter()
            .filter_map(|chain| THORChainName::from_chain(&chain).map(|name| name.chain()))
            .collect::<Vec<Chain>>()
            .into_iter()
            .map(|chain| match chain {
                Chain::Ethereum => SwapChainAsset::Assets(
                    chain,
                    vec![
                        ETHEREUM_USDT.id.clone(),
                        ETHEREUM_USDC.id.clone(),
                        ETHEREUM_DAI.id.clone(),
                        ETHEREUM_WBTC.id.clone(),
                    ],
                ),
                Chain::SmartChain => SwapChainAsset::Assets(chain, vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone()]),
                Chain::AvalancheC => SwapChainAsset::Assets(chain, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
                _ => SwapChainAsset::Assets(chain, vec![]),
            })
            .collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let endpoint = provider
            .get_endpoint(Chain::Thorchain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        let client = ThorChainSwapClient::new(provider);

        //TODO: currently do not support from_asset_id(). As it requires approval for thorchain router
        if request.clone().from_asset.token_id.is_some() {
            return Err(SwapperError::NotSupportedAsset);
        }

        let from_asset = THORChainAsset::from_asset_id(request.clone().from_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(request.clone().to_asset).ok_or(SwapperError::NotSupportedAsset)?;

        let value = self.value_from(request.clone().value, from_asset.decimals as i32);
        let fee = request.options.clone().fee.unwrap_or_default().thorchain;

        let quote = client
            .get_quote(
                endpoint.as_str(),
                from_asset.clone(),
                to_asset.clone(),
                value.to_string(),
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps.into(),
            )
            .await?;

        let to_value = self.value_to(quote.expected_amount_out, to_asset.decimals as i32);

        let quote = SwapQuote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: quote.inbound_address.unwrap_or_default(),
                    gas_estimate: None,
                }],
                suggested_slippage_bps: None,
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let fee = quote.request.options.clone().fee.unwrap_or_default().thorchain;

        let to_asset = THORChainAsset::from_asset_id(quote.clone().request.to_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let memo = to_asset
            .get_memo(
                quote.request.destination_address.clone(),
                QUOTE_MINIMUM,
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps,
            )
            .unwrap();

        let to = quote.data.routes.first().unwrap().route_data.clone();
        let data: String = self.data(quote.request.from_asset.clone().chain, memo);

        let data = SwapQuoteData {
            to,
            value: quote.request.value.clone(),
            data,
        };

        Ok(data)
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let endpoint = provider
            .get_endpoint(Chain::Thorchain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        let client = ThorChainSwapClient::new(provider);

        let status = client.get_transaction_status(&endpoint, transaction_hash).await?;

        Ok(status.observed_tx.status == "done")
    }
}
