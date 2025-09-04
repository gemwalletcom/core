use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{
    client::PaybisClient,
    mapper::{map_order_from_response, map_webhook_order_id},
};
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};

#[async_trait]
impl FiatProvider for PaybisClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(request_map.symbol, request.fiat_currency.to_uppercase(), request.fiat_amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(Box::new(FiatError::UnsupportedState("No payment methods available".to_string())));
        }

        Ok(self.get_buy_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_sell_quote(request_map.symbol, request.fiat_currency.to_uppercase(), request.crypto_amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(Box::new(FiatError::UnsupportedState("No payment methods available".to_string())));
        }

        Ok(self.get_sell_fiat_quote(request, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_assets().await?;

        let assets = response
            .meta
            .currencies
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();

        println!("assets: {:#?}", assets);

        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        let transaction = response.transactions.into_iter().next().ok_or("Transaction not found")?;
        map_order_from_response(transaction)
    }

    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        map_webhook_order_id(data)
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{model::FiatMapping, FiatProvider};
    use primitives::FiatBuyQuote;

    #[tokio::test]
    async fn test_paybis_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();

        let request = FiatBuyQuote::mock();
        let mut mapping = FiatMapping::mock();
        mapping.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_buy_quote(&client, request, mapping).await?;

        println!("Paybis buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "paybis");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_paybis_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());
        println!("Found {} Paybis assets", assets.len());

        if let Some(asset) = assets.first() {
            assert!(!asset.id.is_empty());
            assert!(!asset.symbol.is_empty());
            println!("Sample Paybis asset: {:?}", asset);
        }

        Ok(())
    }
}
