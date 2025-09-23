use crate::{
    FiatProvider,
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
};
use async_trait::async_trait;
use std::error::Error;

use super::models::country::country_status;
use super::{
    client::PaybisClient,
    mapper::{map_assets_with_limits, map_process_webhook},
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
            .get_buy_quote(request_map.symbol, request.fiat_currency.as_ref().to_uppercase(), request.fiat_amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(Box::new(FiatError::UnsupportedState("No payment methods available".to_string())));
        }

        Ok(self.get_buy_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_sell_quote(request_map.symbol, request.fiat_currency.as_ref().to_uppercase(), request.crypto_amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(Box::new(FiatError::UnsupportedState("No payment methods available".to_string())));
        }

        Ok(self.get_sell_fiat_quote(request, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = PaybisClient::get_assets(self).await?;
        let limits = PaybisClient::get_payment_method_limits(self).await?;

        Ok(map_assets_with_limits(assets.meta.currencies, &limits))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        let countries = country_status()
            .iter()
            .map(|(alpha2, is_allowed)| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: alpha2.to_string(),
                is_allowed: *is_allowed,
            })
            .collect();

        Ok(countries)
    }

    async fn get_order_status(&self, _order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        Ok(map_process_webhook(data))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::currency::Currency;
    use primitives::{Chain, FiatBuyQuote};
    use streamer::FiatWebhook;

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
        let result = FiatProvider::get_assets(&client).await?;

        assert!(!result.is_empty());

        let expected_assets = vec![
            ("USDT-TRC20", Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")),
            ("USDT-SOL", Chain::Solana, Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")),
            ("USDT-POLYGON", Chain::Polygon, Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F")),
            ("USDT", Chain::Ethereum, Some("0xdAC17F958D2ee523a2206206994597C13D831ec7")),
            ("USDC-SOL", Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")),
            ("USDC-POLYGON", Chain::Polygon, Some("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")),
            ("USDC-BASE", Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")),
            ("USDC", Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")),
            ("TRX", Chain::Tron, None),
            ("XRP", Chain::Xrp, None),
        ];

        for (symbol, expected_chain, expected_token_id) in expected_assets {
            let asset = result.iter().find(|asset| asset.symbol == symbol);
            assert!(asset.is_some(), "{} asset should exist", symbol);

            if let Some(asset) = asset {
                assert_eq!(asset.chain, Some(expected_chain));
                assert_eq!(asset.token_id.as_deref(), expected_token_id);

                println!("{} asset: {:?}", symbol, asset);
            }
        }

        let usdt_trc20_asset = result.iter().find(|asset| asset.symbol == "USDT-TRC20");
        if let Some(asset) = usdt_trc20_asset {
            assert!(!asset.buy_limits.is_empty(), "USDT-TRC20 should have buy limits");
            let usd_buy_limit = asset.buy_limits.iter().find(|limit| limit.currency == Currency::USD);
            if let Some(limit) = usd_buy_limit {
                assert!(limit.min_amount > Some(1.0));
                assert!(limit.max_amount > Some(1000.0));
            }
        }

        let assets_with_limits = result.iter().filter(|a| !a.buy_limits.is_empty()).count();
        println!("Found {} assets with {} having buy limits", result.len(), assets_with_limits);

        Ok(())
    }

    #[tokio::test]
    async fn test_paybis_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());

        let us_country = countries.iter().find(|c| c.alpha2 == "US").unwrap();
        assert!(us_country.is_allowed);
        assert_eq!(us_country.provider, "paybis");

        let ly_country = countries.iter().find(|c| c.alpha2 == "LY").unwrap();
        assert!(!ly_country.is_allowed);
        assert_eq!(ly_country.provider, "paybis");

        Ok(())
    }

    #[tokio::test]
    async fn test_process_webhook_verification_maps_to_none() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let verification_webhook: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_no_changes.json"))?;

        let result = client.process_webhook(verification_webhook).await?;
        assert!(matches!(result, FiatWebhook::None), "Verification webhooks should map to FiatWebhook::None");

        Ok(())
    }

    #[tokio::test]
    async fn test_process_webhook_transaction() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let transaction_webhook: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json"))?;

        let result = client.process_webhook(transaction_webhook).await?;
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }

        Ok(())
    }
}
