use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::models::{Data, WebhookOrderId},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{client::MoonPayClient, mapper::map_order};
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(request_map.symbol.to_lowercase(), request.fiat_currency.to_lowercase(), request.fiat_amount)
            .await?;

        if quote.total_amount > request.fiat_amount {
            return Err(Box::new(FiatError::MinimumAmount(quote.total_amount)));
        }

        Ok(self.get_buy_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(&request.ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_sell_allowed {
            return Err(FiatError::FiatSellNotAllowed.into());
        }
        let quote = self
            .get_sell_quote(request_map.symbol.to_lowercase(), request.fiat_currency.to_lowercase(), request.crypto_amount)
            .await?;

        Ok(self.get_sell_fiat_quote(request, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
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
                alpha2: x.alpha2,
                is_allowed: x.is_allowed,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.get_any_transaction(order_id).await?;
        map_order(payload)
    }

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<WebhookOrderId>>(data)?.data;
        Ok(payload.id)
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{model::FiatMapping, FiatProvider};
    use primitives::FiatBuyQuote;

    #[tokio::test]
    async fn test_moonpay_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();

        let request = FiatBuyQuote::mock();
        let mut mapping = FiatMapping::mock();
        mapping.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_buy_quote(&client, request, mapping).await?;

        println!("MoonPay buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "moonpay");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_moonpay_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());
        println!("Found {} MoonPay assets", assets.len());

        if let Some(asset) = assets.first() {
            assert!(!asset.id.is_empty());
            assert!(!asset.symbol.is_empty());
            println!("Sample MoonPay asset: {:?}", asset);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_moonpay_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} MoonPay countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, "moonpay");
            assert!(!country.alpha2.is_empty());
            println!("Sample MoonPay country: {:?}", country);
        }

        Ok(())
    }
}
