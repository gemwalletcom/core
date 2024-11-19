mod asset;
mod chain;
mod client;
mod model;

use asset::THORChainAsset;
use chain::THORChainName;
use num_bigint::BigInt;
use std::str::FromStr;

use crate::network::AlienProvider;
use crate::swapper::thorchain::client::ThorChainSwapClient;
use crate::swapper::GemSwapProvider;
use async_trait::async_trait;
use primitives::Chain;
use std::sync::Arc;

use super::{ApprovalType, FetchQuoteData, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError};

#[derive(Debug, Default)]
pub struct ThorChain {}

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;

impl ThorChain {
    fn data(&self, chain: Chain, memo: String) -> String {
        match chain {
            Chain::Thorchain | Chain::Litecoin | Chain::Doge | Chain::Bitcoin | Chain::Cosmos => memo,
            _ => hex::encode(memo.as_bytes()),
        }
    }

    fn value_from(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    fn value_to(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }
}

#[async_trait]
impl GemSwapProvider for ThorChain {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Thorchain
    }

    fn supported_chains(&self) -> Vec<Chain> {
        Chain::all()
            .into_iter()
            .filter_map(|chain| THORChainName::from_chain(&chain).map(|name| name.chain()))
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
        let fee = request.options.clone().unwrap_or_default().fee.unwrap_or_default().thorchain;

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
        let fee = quote.request.options.clone().unwrap_or_default().fee.unwrap_or_default().thorchain;

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

        let to = quote.data.routes.first().unwrap().route_type.clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_from() {
        let thorchain = ThorChain::default();

        let value = "1000000000".to_string();

        let result = thorchain.value_from(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("0").unwrap());

        let result = thorchain.value_from(value.clone(), 10);
        assert_eq!(result, BigInt::from_str("10000000").unwrap());

        let result = thorchain.value_from(value.clone(), 6);
        assert_eq!(result, BigInt::from_str("100000000000").unwrap());

        let result = thorchain.value_from(value.clone(), 8);
        assert_eq!(result, BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        let thorchain = ThorChain::default();

        let value = "10000000".to_string();

        let result = thorchain.value_to(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("100000000000000000").unwrap());

        let result = thorchain.value_to(value.clone(), 10);
        assert_eq!(result, BigInt::from(1000000000));

        let result = thorchain.value_to(value.clone(), 6);
        assert_eq!(result, BigInt::from(100000));

        let result = thorchain.value_to(value.clone(), 8);
        assert_eq!(result, BigInt::from(10000000));
    }
}
