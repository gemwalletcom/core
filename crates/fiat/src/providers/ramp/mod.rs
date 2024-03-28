use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote};

use crate::model::{FiatMapping, FiatProvider, FiatProviderAsset};

use self::model::QuoteRequest;
use async_trait::async_trait;

use super::RampClient;

pub mod client;
pub mod model;

#[async_trait]
impl FiatProvider for RampClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;

        let crypto_asset_symbol = format!(
            "{}_{}",
            request_map.network.unwrap_or_default(),
            request_map.symbol,
        );

        if !assets
            .iter()
            .any(|x| x.crypto_asset_symbol() == crypto_asset_symbol)
        {
            return Err("asset not supported".into());
        }

        let payload = QuoteRequest {
            crypto_asset_symbol,
            fiat_currency: request.clone().fiat_currency,
            fiat_value: request.fiat_amount,
        };
        let quote = self.get_client_quote(payload).await?;

        Ok(self.get_fiat_quote(request.clone(), quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets("USD".to_string(), "127.0.0.0".to_string())
            .await?
            .assets
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn get_transactions(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}
