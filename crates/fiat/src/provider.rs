use std::sync::Arc;

use crate::model::{FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use primitives::fiat_quote_request::FiatSellRequest;
use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction};

#[async_trait]
pub trait FiatProvider {
    fn name(&self) -> FiatProviderName;
    async fn get_buy_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_sell_quote(
        &self,
        request: FiatSellRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;

    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> FiatProvider for Arc<T>
where
    T: FiatProvider + ?Sized,
{
    fn name(&self) -> FiatProviderName {
        (**self).name()
    }

    async fn get_buy_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_buy_quote(request, request_map).await
    }

    async fn get_sell_quote(
        &self,
        request: FiatSellRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_sell_quote(request, request_map).await
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_assets().await
    }

    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        (**self).webhook(data).await
    }
}
