use crate::{
    FiatProvider,
    model::{FiatMapping, FiatProviderAsset},
    providers::banxa::mapper::map_asset_with_limits,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatSellQuote};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuote, FiatTransaction};
use std::error::Error;
use streamer::FiatWebhook;

use super::{
    client::BanxaClient,
    mapper::map_order,
    models::{ORDER_TYPE_SELL, Webhook},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                &request_map.clone().symbol,
                &request_map.clone().network.unwrap_or_default(),
                request.fiat_currency.as_ref(),
                request.fiat_amount,
            )
            .await?;

        Ok(self.get_fiat_buy_quote(request, request_map, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        // v2/payment-methods/sell
        let method = self
            .get_payment_methods(ORDER_TYPE_SELL)
            .await?
            .into_iter()
            .find(|x| x.supported_fiats.contains(&request.fiat_currency.as_ref().to_string()))
            .ok_or("Payment method not found")?;

        let quote = self
            .get_quote_sell(
                &method.id,
                &request_map.symbol,
                &request_map.clone().network.unwrap_or_default(),
                request.fiat_currency.as_ref(),
                request.crypto_amount,
            )
            .await?;
        Ok(self.get_fiat_sell_quote(request, request_map, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let (assets, buy_fiat_currencies, sell_fiat_currencies) =
            tokio::try_join!(self.get_assets_buy(), self.get_fiat_currencies("buy"), self.get_fiat_currencies("sell"))?;

        let assets = assets
            .into_iter()
            .flat_map(|x| map_asset_with_limits(x, &buy_fiat_currencies, &sell_fiat_currencies))
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.id,
                is_allowed: true,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let order = self.get_order(order_id).await?;
        map_order(order)
    }

    // https://docs.banxa.com/docs/webhooks
    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        let order_id = serde_json::from_value::<Webhook>(data)?.order_id;
        Ok(FiatWebhook::OrderId(order_id))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::FiatBuyQuote;

    #[tokio::test]
    async fn test_banxa_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();

        let request = FiatBuyQuote::mock();
        let mut mapping = FiatMapping::mock();
        mapping.network = Some("BTC".to_string());

        let quote = FiatProvider::get_buy_quote(&client, request, mapping).await?;

        println!("Banxa buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "banxa");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_banxa_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());

        let assets_with_buy_limits = assets.iter().filter(|a| !a.buy_limits.is_empty()).count();
        assert!(assets_with_buy_limits > 0);

        if let Some(asset_with_limits) = assets.iter().find(|a| !a.buy_limits.is_empty()) {
            let first_limit = &asset_with_limits.buy_limits[0];
            assert!(first_limit.min_amount.is_some() || first_limit.max_amount.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_banxa_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} Banxa countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, "banxa");
            assert!(!country.alpha2.is_empty());
            println!("Sample Banxa country: {:?}", country);
        }

        Ok(())
    }
}
