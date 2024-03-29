use async_trait::async_trait;
use primitives::{
    FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus,
};

use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};

use super::{client::MercuryoClient, model::Webhook};

#[async_trait]
impl FiatProvider for MercuryoClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                request.fiat_currency.clone(),
                request_map.symbol.clone(),
                request.fiat_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;

        Ok(self.get_fiat_quote(request, request_map.clone(), quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    // full transaction: https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#22-callbacks-response-body
    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::from_value::<Webhook>(data)?.payload.data;

        // https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#3-transaction-status-types
        let status = match data.status.as_str() {
            "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
            "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
            "paid" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let transaction = FiatTransaction {
            asset_id: None,
            symbol: data.currency,
            provider_id: Self::NAME.id(),
            transaction_id: data.id,
            status,
            fiat_amount: data.amount.parse::<f64>()?,
            fiat_currency: data.fiat_currency,
        };

        Ok(transaction)
    }
}
