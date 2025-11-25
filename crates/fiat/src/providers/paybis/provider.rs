use crate::{
    FiatProvider,
    error::FiatQuoteError,
    model::{FiatMapping, FiatProviderAsset},
};
use async_trait::async_trait;
use std::error::Error;

use super::models::country::country_status;
use super::{
    client::PaybisClient,
    mapper::{map_assets, map_process_webhook},
};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatTransaction};
use streamer::FiatWebhook;

#[async_trait]
impl FiatProvider for PaybisClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = PaybisClient::get_assets(self).await?;
        Ok(map_assets(assets.meta.currencies))
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

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(request_map.asset_symbol.symbol, request.currency.to_uppercase(), request.amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(FiatQuoteError::UnsupportedState("No payment methods available".to_string()).into());
        }

        let payment_method = quote.payment_methods.first().unwrap();
        let crypto_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        Ok(FiatQuoteResponse::new(quote.id, request.amount, crypto_amount))
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_sell_quote(request_map.asset_symbol.symbol, request.currency.to_uppercase(), request.amount)
            .await?;

        if quote.payment_methods.is_empty() {
            return Err(FiatQuoteError::UnsupportedState("No payment methods available".to_string()).into());
        }

        let payment_method = quote.payment_methods.first().unwrap();
        let fiat_amount: f64 = payment_method.amount_to.amount.parse().unwrap_or(0.0);

        Ok(FiatQuoteResponse::new(quote.id, fiat_amount, request.amount))
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let is_buy = matches!(data.quote.quote_type, primitives::FiatQuoteType::Buy);
        let redirect_url = self
            .get_redirect_url(
                &data.wallet_address,
                &data.quote.fiat_currency,
                &data.asset_symbol.symbol,
                &data.quote.id,
                is_buy,
                &data.ip_address,
                &data.locale,
            )
            .await?;

        Ok(FiatQuoteUrl { redirect_url })
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::currency::Currency;
    use primitives::{Chain, FiatBuyQuote, FiatQuoteRequest};
    use streamer::FiatWebhook;

    #[tokio::test]
    async fn test_paybis_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();

        let request = FiatBuyQuote::mock();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_buy_quote_old(&client, request, mapping).await?;

        println!("Paybis buy quote: {:?}", quote);
        assert_eq!(quote.provider.id, "paybis");
        assert_eq!(quote.fiat_currency, "USD");
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_paybis_get_sell_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();

        let request = FiatQuoteRequest::mock_sell();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_quote_sell(&client, request, mapping).await?;

        println!("Paybis sell quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert!(quote.fiat_amount > 0.0);
        assert_eq!(quote.crypto_amount, 0.001);

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
            assert!(usd_buy_limit.is_some(), "Should have USD limit with Card payment type");
        }

        println!("Found {} assets", result.len());

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
