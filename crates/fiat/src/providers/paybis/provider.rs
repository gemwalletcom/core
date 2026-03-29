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
    mapper::{map_assets, map_process_webhook, supported_payment_methods},
};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatTransactionUpdate, PaymentType};
use streamer::FiatWebhook;

#[async_trait]
impl FiatProvider for PaybisClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn payment_methods(&self) -> Vec<PaymentType> {
        supported_payment_methods()
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let buy_assets = PaybisClient::get_buy_assets(self).await?;
        let sell_assets = PaybisClient::get_sell_assets(self).await?;
        let buy_currencies = buy_assets.meta.currencies;
        let sell_currencies = sell_assets.get_crypto_codes();
        Ok(map_assets(buy_currencies, sell_currencies))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        let countries = country_status()
            .iter()
            .map(|(alpha2, is_allowed)| FiatProviderCountry {
                provider: Self::NAME,
                alpha2: alpha2.to_string(),
                is_allowed: *is_allowed,
            })
            .collect();

        Ok(countries)
    }

    async fn get_order_status(&self, _order_id: &str) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        Ok(map_process_webhook(data)?)
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self.get_buy_quote(request_map.asset_symbol.symbol, request.currency.to_uppercase(), request.amount).await?;

        let payment_method = quote
            .payment_methods
            .first()
            .ok_or_else(|| FiatQuoteError::UnsupportedState("No payment methods available".to_string()))?;
        let crypto_amount: f64 = payment_method.amount_to.amount.parse()?;

        Ok(FiatQuoteResponse::new(quote.id, request.amount, crypto_amount))
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_sell_quote(request_map.asset_symbol.symbol, request.currency.to_uppercase(), request.amount)
            .await?;

        let payout_method = quote
            .payout_methods
            .first()
            .ok_or_else(|| FiatQuoteError::UnsupportedState("No payout methods available".to_string()))?;
        let crypto_amount: f64 = payout_method.amount_from.amount.parse()?;

        Ok(FiatQuoteResponse::new(quote.id, request.amount, crypto_amount))
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let is_buy = match data.quote.quote_type {
            primitives::FiatQuoteType::Buy => true,
            primitives::FiatQuoteType::Sell => false,
        };
        let (redirect_url, request_id) = self
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

        Ok(FiatQuoteUrl {
            redirect_url,
            provider_transaction_id: Some(request_id),
        })
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::asset_constants::{
        BASE_USDC_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID, POLYGON_USDC_TOKEN_ID, POLYGON_USDT_TOKEN_ID, SOLANA_USDC_TOKEN_ID, SOLANA_USDT_TOKEN_ID,
        TRON_USDT_TOKEN_ID,
    };
    use primitives::currency::Currency;
    use primitives::{Chain, FiatProviderName, FiatQuoteRequest};
    use streamer::FiatWebhook;

    #[tokio::test]
    async fn test_paybis_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();

        let request = FiatQuoteRequest::mock();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_quote_buy(&client, request.clone(), mapping).await?;

        println!("Paybis buy quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, request.amount);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Paybis does not currently support sell for most assets"]
    async fn test_paybis_get_sell_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();

        let request = FiatQuoteRequest::mock_sell();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.symbol = "ETH".to_string();
        mapping.asset_symbol.network = Some("ethereum".to_string());

        let quote = FiatProvider::get_quote_sell(&client, request.clone(), mapping).await?;

        println!("Paybis sell quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert_eq!(quote.fiat_amount, request.amount);
        assert!(quote.crypto_amount > 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_paybis_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let result = FiatProvider::get_assets(&client).await?;

        assert!(!result.is_empty());

        let expected_assets = vec![
            ("USDT-TRC20", Chain::Tron, Some(TRON_USDT_TOKEN_ID.to_string())),
            ("USDT-SOL", Chain::Solana, Some(SOLANA_USDT_TOKEN_ID.to_string())),
            ("USDT-POLYGON", Chain::Polygon, Some(POLYGON_USDT_TOKEN_ID.to_string())),
            ("USDT", Chain::Ethereum, Some(ETHEREUM_USDT_TOKEN_ID.to_string())),
            ("USDC-SOL", Chain::Solana, Some(SOLANA_USDC_TOKEN_ID.to_string())),
            ("USDC-POLYGON", Chain::Polygon, Some(POLYGON_USDC_TOKEN_ID.to_string())),
            ("USDC-BASE", Chain::Base, Some(BASE_USDC_TOKEN_ID.to_string())),
            ("USDC", Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.to_string())),
            ("TRX", Chain::Tron, None),
            ("XRP", Chain::Xrp, None),
        ];

        for (symbol, expected_chain, expected_token_id) in expected_assets {
            let asset = result.iter().find(|asset| asset.symbol == symbol);
            assert!(asset.is_some(), "{} asset should exist", symbol);

            if let Some(asset) = asset {
                assert_eq!(asset.chain, Some(expected_chain));
                assert_eq!(asset.token_id, expected_token_id);

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
        assert_eq!(us_country.provider, FiatProviderName::Paybis);

        let ly_country = countries.iter().find(|c| c.alpha2 == "LY").unwrap();
        assert!(!ly_country.is_allowed);
        assert_eq!(ly_country.provider, FiatProviderName::Paybis);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_webhook_verification_maps_to_none() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let verification_webhook: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_no_changes.json"))?;

        let result = client.process_webhook(verification_webhook).await?;
        match result {
            FiatWebhook::None => {}
            _ => panic!("Verification webhooks should map to FiatWebhook::None"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_process_webhook_transaction() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_paybis_test_client();
        let transaction_webhook: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json"))?;

        let result = client.process_webhook(transaction_webhook).await?;
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.transaction_id, "3b388a91-d1fa-456e-b94a");
            assert_eq!(transaction.provider_transaction_id, Some("PB21095868675TX1".to_string()));
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }

        Ok(())
    }
}
