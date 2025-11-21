use std::sync::Arc;

use crate::model::{FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use primitives::{
    FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuoteOld, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatSellQuote,
    FiatTransaction,
};
use streamer::FiatWebhook;

#[async_trait]
pub trait FiatProvider: Send + Sync {
    fn name(&self) -> FiatProviderName;

    async fn get_buy_quote_old(&self, _request: FiatBuyQuote, _request_map: FiatMapping) -> Result<FiatQuoteOld, Box<dyn std::error::Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn get_sell_quote_old(&self, _request: FiatSellQuote, _request_map: FiatMapping) -> Result<FiatQuoteOld, Box<dyn std::error::Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>>;

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> FiatProvider for Arc<T>
where
    T: FiatProvider + ?Sized,
{
    fn name(&self) -> FiatProviderName {
        (**self).name()
    }

    async fn get_buy_quote_old(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuoteOld, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_buy_quote_old(request, request_map).await
    }

    async fn get_sell_quote_old(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuoteOld, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_sell_quote_old(request, request_map).await
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_assets().await
    }
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_countries().await
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_order_status(order_id).await
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        (**self).process_webhook(data).await
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_buy(request, request_map).await
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_sell(request, request_map).await
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_url(data).await
    }
}
