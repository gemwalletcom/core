use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatSellQuote};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuote, FiatTransaction};
use std::error::Error;

use super::{
    client::BanxaClient,
    mapper::map_order,
    model::{Webhook, ORDER_TYPE_SELL},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                &request_map.clone().symbol,
                &request_map.clone().network.unwrap_or_default(),
                &request.fiat_currency,
                request.fiat_amount,
            )
            .await?;

        Ok(self.get_fiat_buy_quote(request, request_map, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        // v2/payment-methods/sell
        let method = self
            .get_payment_methods(ORDER_TYPE_SELL)
            .await?
            .into_iter()
            .find(|x| x.supported_fiats.contains(&request.fiat_currency))
            .ok_or("Payment method not found")?;

        let quote = self
            .get_quote_sell(
                &method.id,
                &request_map.symbol,
                &request_map.clone().network.unwrap_or_default(),
                &request.fiat_currency,
                request.crypto_amount,
            )
            .await?;
        Ok(self.get_fiat_sell_quote(request, request_map, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets_buy()
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
                alpha2: x.id,
                is_allowed: true,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let order = self.get_order(order_id).await?;
        map_order(order)
    }

    // https://docs.banxa.com/docs/webhooks
    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::from_value::<Webhook>(data)?.order_id)
    }
}
