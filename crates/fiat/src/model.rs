use async_trait::async_trait;
use primitives::{
    fiat_provider::FiatProviderName, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
    Chain,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub struct FiatRequestMap {
    pub crypto_currency: String,
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatRates {
    pub rates: Vec<storage::models::FiatRate>,
}

// mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatMapping {
    pub symbol: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderAsset {
    pub chain: Chain,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;

#[async_trait]
pub trait FiatClient {
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
impl<T: Send + Sync> FiatClient for Arc<T>
where
    T: FiatClient + ?Sized,
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
