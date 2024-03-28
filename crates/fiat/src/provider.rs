use std::sync::Arc;

use async_trait::async_trait;
use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote};

use crate::model::{FiatMapping, FiatProviderAsset};

#[async_trait]
pub trait FiatProvider {
    fn name(&self) -> FiatProviderName;
    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> FiatProvider for Arc<T>
where
    T: FiatProvider + ?Sized,
{
    fn name(&self) -> FiatProviderName {
        (**self).name()
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_quote(request, request_map).await
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_assets().await
    }
}
