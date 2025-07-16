use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{client::PaybisClient, model::PaybisWebhook};
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus};

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

        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<PaybisWebhook>(data)?;

        let transaction_type = FiatQuoteType::Buy;

        let status = match payload.status.as_str() {
            "pending" => FiatTransactionStatus::Pending,
            "failed" | "cancelled" => FiatTransactionStatus::Failed,
            "completed" | "success" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown(payload.status),
        };

        // We don't have enough information to determine the exact asset from the webhook
        // The crypto_currency field doesn't specify which chain the asset is on
        let asset_id = None;

        let transaction = FiatTransaction {
            asset_id,
            transaction_type,
            symbol: payload.crypto_currency.clone(),
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.id,
            status,
            country: payload.country,
            fiat_amount: payload.fiat_amount,
            fiat_currency: payload.fiat_currency.to_uppercase(),
            transaction_hash: payload.transaction_hash,
            address: payload.wallet_address,
            fee_provider: payload.service_fee,
            fee_network: payload.network_fee,
            fee_partner: payload.partner_fee,
        };

        Ok(transaction)
    }
}
