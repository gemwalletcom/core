use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::model::{Data, Webhook},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{client::MoonPayClient, model::FiatCurrencyType};
use primitives::{
    AssetId, FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus,
};

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

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<Webhook>>(data)?.data;

        let asset = payload.clone().currency.unwrap_or(payload.clone().base_currency);
        let fiat_currency = payload.clone().quote_currency.unwrap_or(payload.clone().base_currency);
        let asset = Self::map_asset(asset).unwrap();
        let asset_id = AssetId::from(asset.chain.unwrap(), asset.token_id);

        let transaction_type = if payload.clone().base_currency.currency_type == FiatCurrencyType::Fiat {
            FiatQuoteType::Buy
        } else {
            FiatQuoteType::Sell
        };
        let currency_amount = match transaction_type {
            FiatQuoteType::Buy => payload.base_currency_amount.unwrap_or_default(),
            FiatQuoteType::Sell => payload.quote_currency_amount.unwrap_or_default(),
        };

        let status = match payload.status.as_str() {
            "pending" | "waitingForDeposit" => FiatTransactionStatus::Pending,
            "failed" => FiatTransactionStatus::Failed,
            "completed" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown(payload.status),
        };
        let fee_provider = payload.fee_amount.unwrap_or_default();
        let fee_network = payload.network_fee_amount.unwrap_or_default();
        let fee_partner = payload.extra_fee_amount.unwrap_or_default();
        let fiat_amount = currency_amount + fee_provider + fee_network + fee_partner;

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            transaction_type,
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.id,
            status,
            fiat_amount,
            fiat_currency: fiat_currency.code.to_uppercase(),
            transaction_hash: payload.crypto_transaction_id,
            address: payload.wallet_address,
            fee_provider: payload.fee_amount,
            fee_network: payload.network_fee_amount,
            fee_partner: payload.extra_fee_amount,
        };

        Ok(transaction)
    }
}
