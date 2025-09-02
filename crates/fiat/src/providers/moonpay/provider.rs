use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::models::{Data, WebhookOrderId},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::{client::MoonPayClient, mapper::map_order};
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};

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

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.get_any_transaction(order_id).await?;
        map_order(payload)
    }

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<WebhookOrderId>>(data)?.data;
        Ok(payload.id)
    }
}
