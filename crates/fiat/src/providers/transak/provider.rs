use super::{
    client::TransakClient,
    mapper::map_order_from_response,
    models::{Data, WebhookPayload},
};
use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatSellQuote, FiatTransaction};
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
            .response
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .response
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.alpha2,
                is_allowed: x.is_allowed,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.get_transaction(order_id).await?;
        map_order_from_response(response)
    }

    async fn webhook_order_id(&self, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let encrypted_data = serde_json::from_value::<Data<String>>(data)?;
        let decoded_payload = self
            .decode_jwt_content(&encrypted_data.data)
            .map_err(|e| format!("Failed to decode Transak JWT: {}", e))?;
        let webhook_payload = serde_json::from_str::<WebhookPayload>(&decoded_payload)?;
        Ok(webhook_payload.webhook_data.id)
    }
}
