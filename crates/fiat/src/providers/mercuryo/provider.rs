use crate::{
    model::{FiatMapping, FiatProviderAsset},
    providers::mercuryo::mapper::{map_asset_limits, map_asset_with_limits},
    FiatProvider,
};
use async_trait::async_trait;
use futures::future;
use primitives::currency::Currency;
use primitives::{FiatBuyQuote, FiatSellQuote};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuote, FiatTransaction};
use std::error::Error;
use streamer::FiatWebhook;

use super::{client::MercuryoClient, mapper::map_order_from_response, models::Webhook};

#[async_trait]
impl FiatProvider for MercuryoClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                request.fiat_currency.as_ref().to_string(),
                request_map.symbol.clone(),
                request.fiat_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;

        Ok(self.get_fiat_buy_quote(request, request_map.clone(), quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_quote_sell(
                request.fiat_currency.as_ref().to_string(),
                request_map.symbol.clone(),
                request.crypto_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;
        Ok(self.get_fiat_sell_quote(request, request_map, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let currencies = self.get_currencies().await?;
        let currency = Currency::USD;

        let assets_with_limits = future::join_all(currencies.config.crypto_currencies.into_iter().map(|asset| {
            let fiat_payment_methods = currencies.fiat_payment_methods.clone();
            let currency = currency.clone();
            async move {
                match self.get_currency_limits(asset.currency.clone(), currency.as_ref().to_string()).await {
                    Ok(response) => (
                        asset,
                        map_asset_limits(response.data.get(currency.as_ref()), currency.clone(), &fiat_payment_methods),
                    ),
                    Err(_) => (asset, map_asset_limits(None, currency, &fiat_payment_methods)),
                }
            }
        }))
        .await;

        Ok(assets_with_limits
            .into_iter()
            .filter_map(|(asset, limits)| map_asset_with_limits(asset, limits.clone(), limits))
            .collect())
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .data
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.to_uppercase(),
                is_allowed: true,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        let transaction = response.data.into_iter().next().ok_or("Transaction not found")?;
        map_order_from_response(transaction)
    }

    // full transaction: https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#22-callbacks-response-body
    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        let webhook_data = serde_json::from_value::<Webhook>(data)?.data;
        let order_id = webhook_data.merchant_transaction_id.unwrap_or(webhook_data.id);
        Ok(FiatWebhook::OrderId(order_id))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{model::FiatMapping, FiatProvider};
    use primitives::FiatBuyQuote;

    #[tokio::test]
    async fn test_mercuryo_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_mercuryo_test_client();

        let request = FiatBuyQuote::mock();
        let mapping = FiatMapping::mock();

        let quote = FiatProvider::get_buy_quote(&client, request, mapping).await?;

        println!("Mercuryo buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "mercuryo");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_mercuryo_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_mercuryo_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());

        let assets_with_limits = assets.iter().filter(|a| !a.buy_limits.is_empty()).count();
        assert!(assets_with_limits > 0);

        if let Some(asset) = assets.iter().find(|a| !a.buy_limits.is_empty()) {
            assert_eq!(asset.buy_limits.len(), asset.sell_limits.len());
            assert!(asset.buy_limits[0].min_amount.is_some() || asset.buy_limits[0].max_amount.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_mercuryo_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_mercuryo_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} Mercuryo countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, "mercuryo");
            assert!(!country.alpha2.is_empty());
            println!("Sample Mercuryo country: {:?}", country);
        }

        Ok(())
    }
}
