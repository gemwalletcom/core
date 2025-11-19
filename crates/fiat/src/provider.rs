use std::sync::Arc;

use crate::model::{FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use primitives::{
    FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatQuoteDataRequest, FiatQuoteDataResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatSellQuote,
    FiatTransaction,
};
use streamer::FiatWebhook;

#[async_trait]
pub trait FiatProvider {
    fn name(&self) -> FiatProviderName;
    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>>;

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_quote_buy_data(
        &self,
        request: FiatQuoteDataRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuoteDataResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_sell_data(
        &self,
        request: FiatQuoteDataRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuoteDataResponse, Box<dyn std::error::Error + Send + Sync>>;
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

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_buy_quote(request, request_map).await
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_sell_quote(request, request_map).await
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

    async fn get_quote_buy_data(
        &self,
        request: FiatQuoteDataRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuoteDataResponse, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_buy_data(request, request_map).await
    }

    async fn get_quote_sell_data(
        &self,
        request: FiatQuoteDataRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuoteDataResponse, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_sell_data(request, request_map).await
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote_url(data).await
    }
}
