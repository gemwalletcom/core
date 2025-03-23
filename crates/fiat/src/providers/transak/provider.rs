use super::{client::TransakClient, model::WebhookPayload};
use crate::{
    model::{FiatMapping, FiatProviderAsset},
    providers::transak::model::WebhookEncryptedData,
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus};
use std::error::Error;

#[async_trait]
impl FiatProvider for TransakClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(
                request_map.symbol.clone(),
                request.fiat_currency.clone(),
                request.fiat_amount,
                request_map.network.unwrap_or_default(),
                request.ip_address.clone(),
            )
            .await?;
        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, _request: FiatSellQuote, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<WebhookEncryptedData>(data)?;
        let payload = self.decode_jwt_content(&payload.data).unwrap();
        let payload = serde_json::from_str::<WebhookPayload>(&payload)?.webhook_data;

        let status = match payload.status.as_str() {
            "ORDER_PAYMENT_VERIFYING" | "PAYMENT_DONE_MARKED_BY_USER" | "PENDING_DELIVERY_FROM_TRANSAK" | "AWAITING_PAYMENT_FROM_USER" => {
                FiatTransactionStatus::Pending
            }
            "EXPIRED" | "FAILED" | "CANCELLED" | "REFUNDED" => FiatTransactionStatus::Failed,
            "COMPLETED" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };
        let transaction_type = FiatQuoteType::Buy;

        let transaction = FiatTransaction {
            asset_id: None,
            transaction_type,
            symbol: payload.crypto_currency,
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.id,
            status,
            fiat_amount: payload.fiat_amount,
            fiat_currency: payload.fiat_currency,
            transaction_hash: payload.transaction_hash,
            address: Some(payload.wallet_address),
            fee_provider: payload.conversion_price_data.as_ref().and_then(|data| data.fee("transak_fee")),
            fee_network: payload.conversion_price_data.as_ref().and_then(|data| data.fee("network_fee")),
            fee_partner: payload.conversion_price_data.as_ref().and_then(|data| data.fee("partner_fee")),
        };
        Ok(transaction)
    }
}
