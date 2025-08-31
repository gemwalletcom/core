use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatSellQuote};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuote, FiatTransaction};
use std::error::Error;

use super::{client::MercuryoClient, mapper::map_order_from_response, model::Webhook};

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

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .data
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.to_uppercase(),
                is_allowed: true,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        let transaction = response.data.into_iter().next().ok_or("Transaction not found")?;
        map_order_from_response(transaction)
    }

    // full transaction: https://github.com/mercuryoio/api-migration-docs/blob/master/Widget_API_Mercuryo_v1.6.md#22-callbacks-response-body
    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let webhook_data = serde_json::from_value::<Webhook>(data)?.data;
        Ok(webhook_data.merchant_transaction_id.unwrap_or(webhook_data.id))
    }
}
