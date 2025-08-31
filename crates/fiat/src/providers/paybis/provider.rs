use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{client::PaybisClient, model::PaybisWebhook, mapper::map_order_from_response};
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
        let transaction = response.transactions.into_iter()
            .next()
            .ok_or("Transaction not found")?;
        map_order_from_response(transaction)
    }

    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<PaybisWebhook>(data)?;
        Ok(payload.id)
    }
}
