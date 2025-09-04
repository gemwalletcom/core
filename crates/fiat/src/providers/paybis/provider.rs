use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{
    client::PaybisClient,
    mapper::{map_order_from_response, map_webhook_to_transaction},
};
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};
use streamer::FiatWebhook;

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

        Ok(response
            .meta
            .currencies
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>())
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        let transaction = response.transactions.into_iter().next().ok_or("Transaction not found")?;
        map_order_from_response(transaction)
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        let transaction = map_webhook_to_transaction(data)?;
        Ok(FiatWebhook::Transaction(transaction))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{model::FiatMapping, FiatProvider};
    use primitives::{Chain, FiatBuyQuote};

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

        let expected_assets = vec![
            ("USDT-TRC20", Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")),
            ("TRX", Chain::Tron, None),
            ("XRP", Chain::Xrp, None),
        ];

        for (symbol, expected_chain, expected_token_id) in expected_assets {
            let asset = assets.iter().find(|asset| asset.symbol == symbol);
            assert!(asset.is_some(), "{} asset should exist", symbol);

            if let Some(asset) = asset {
                assert_eq!(asset.chain, Some(expected_chain));
                assert_eq!(asset.token_id.as_deref(), expected_token_id);
                println!("{} asset: {:?}", symbol, asset);
            }
        }

        Ok(())
    }
}
