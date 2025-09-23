use super::{
    client::TransakClient,
    mapper::map_order_from_response,
    models::{Data, WebhookPayload},
};
use crate::{
    FiatProvider,
    model::{FiatMapping, FiatProviderAsset},
    providers::transak::mapper::map_asset_with_limits,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};
use std::error::Error;
use streamer::FiatWebhook;

#[async_trait]
impl FiatProvider for TransakClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(
                request_map.symbol.clone(),
                request.fiat_currency.as_ref().to_string(),
                request.fiat_amount,
                request_map.network.unwrap_or_default(),
                request.ip_address.clone(),
            )
            .await?;
        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, _request: FiatSellQuote, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let (assets, fiat_currencies) = tokio::try_join!(self.get_supported_assets(), self.get_fiat_currencies())?;
        Ok(assets
            .response
            .into_iter()
            .flat_map(|asset| map_asset_with_limits(asset, &fiat_currencies.response))
            .collect::<Vec<FiatProviderAsset>>())
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .response
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.alpha2,
                is_allowed: x.is_allowed,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        map_order_from_response(response)
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        let encrypted_data = serde_json::from_value::<Data<String>>(data)?;
        let decoded_payload = self
            .decode_jwt_content(&encrypted_data.data)
            .map_err(|e| format!("Failed to decode Transak JWT: {}", e))?;
        let webhook_payload = serde_json::from_str::<WebhookPayload>(&decoded_payload)?;
        Ok(FiatWebhook::OrderId(webhook_payload.webhook_data.id))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::FiatBuyQuote;

    #[tokio::test]
    async fn test_transak_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();

        let request = FiatBuyQuote::mock();
        let mut mapping = FiatMapping::mock();
        mapping.network = Some("mainnet".to_string());

        let quote = FiatProvider::get_buy_quote(&client, request, mapping).await?;

        println!("Transak buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "transak");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_transak_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());
        println!("Found {} Transak assets", assets.len());

        if let Some(asset) = assets.first() {
            assert!(!asset.id.is_empty());
            assert!(!asset.symbol.is_empty());
            println!("Sample Transak asset: {:?}", asset);
        }

        let assets_with_buy_limits = assets.iter().filter(|a| !a.buy_limits.is_empty()).count();
        let assets_with_sell_limits = assets.iter().filter(|a| !a.sell_limits.is_empty()).count();

        println!("Assets with buy limits: {}", assets_with_buy_limits);
        println!("Assets with sell limits: {}", assets_with_sell_limits);

        assert!(assets_with_buy_limits > 0, "Expected at least some assets to have buy limits");

        if let Some(asset_with_limits) = assets.iter().find(|a| !a.buy_limits.is_empty()) {
            println!(
                "Asset with limits: {} has {} buy limits",
                asset_with_limits.symbol,
                asset_with_limits.buy_limits.len()
            );

            let first_limit = &asset_with_limits.buy_limits[0];
            assert!(first_limit.min_amount.is_some() || first_limit.max_amount.is_some());
            println!("Sample limit: {:?}", first_limit);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_transak_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} Transak countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, "transak");
            assert!(!country.alpha2.is_empty());
            println!("Sample Transak country: {:?}", country);
        }

        Ok(())
    }
}
