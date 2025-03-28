use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{fiat_transaction::FiatQuoteType, FiatBuyQuote, FiatSellQuote};
use primitives::{FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus};
use std::error::Error;

use super::{client::MercuryoClient, model::Webhook};

#[async_trait]
impl FiatProvider for MercuryoClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                request.fiat_currency.clone(),
                request_map.symbol.clone(),
                request.fiat_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;

        Ok(self.get_fiat_buy_quote(request, request_map.clone(), quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_quote_sell(
                request.fiat_currency.clone(),
                request_map.symbol.clone(),
                request.crypto_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;
        Ok(self.get_fiat_sell_quote(request, request_map, quote))
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

    // full transaction: https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#22-callbacks-response-body
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::from_value::<Webhook>(data)?.data;

        // https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#3-transaction-status-types
        let status = match data.status.as_str() {
            "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
            "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
            "paid" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };
        let transaction_type = FiatQuoteType::Buy;

        let transaction = FiatTransaction {
            asset_id: None,
            transaction_type,
            symbol: data.currency,
            provider_id: Self::NAME.id(),
            provider_transaction_id: data.merchant_transaction_id.unwrap_or(data.id),
            status,
            fiat_amount: data.fiat_amount,
            fiat_currency: data.fiat_currency,
            transaction_hash: data.tx.clone().and_then(|x| x.id),
            address: data.tx.clone().and_then(|x| x.address),
            fee_provider: data.fee,
            fee_network: None,
            fee_partner: data.partner_fee,
        };

        print!("transaction: {:?}", transaction);

        Ok(transaction)
    }
}
