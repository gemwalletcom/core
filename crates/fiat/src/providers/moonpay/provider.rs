use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::model::{Data, Webhook},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::client::MoonPayClient;
use primitives::{AssetId, FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus};

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(&request.ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_buy_allowed {
            return Err(FiatError::FiatPurchaseNotAllowed.into());
        }

        let quote = self
            .get_buy_quote(request_map.symbol.to_lowercase(), request.fiat_currency.to_lowercase(), request.fiat_amount)
            .await?;

        self.validate_quote(&quote, ip_address_check).await?;

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

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<Webhook>>(data)?;
        let asset = Self::map_asset(payload.data.currency).unwrap();
        let asset_id = AssetId::from(asset.chain.unwrap(), asset.token_id);

        let status = match payload.data.status.as_str() {
            "pending" => FiatTransactionStatus::Pending,
            "failed" => FiatTransactionStatus::Failed,
            "completed" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let currency_amount = payload.data.base_currency_amount;
        let fee_provider = payload.data.fee_amount.unwrap_or_default();
        let fee_network = payload.data.network_fee_amount.unwrap_or_default();
        let fee_partner = payload.data.extra_fee_amount.unwrap_or_default();
        let fiat_amount = currency_amount + fee_provider + fee_network + fee_partner;
        let transaction_type = FiatQuoteType::Buy;

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            transaction_type,
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.data.id,
            status,
            fiat_amount,
            fiat_currency: payload.data.base_currency.code.to_uppercase(),
            transaction_hash: payload.data.crypto_transaction_id,
            address: payload.data.wallet_address,
            fee_provider: payload.data.fee_amount,
            fee_network: payload.data.network_fee_amount,
            fee_partner: payload.data.extra_fee_amount,
        };

        Ok(transaction)
    }
}
