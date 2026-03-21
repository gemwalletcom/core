use std::sync::Arc;

use crate::model::{FiatMapping, FiatProviderAsset};
use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatTransactionUpdate, PaymentType};
use streamer::FiatWebhook;

pub(crate) fn generate_quote_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[async_trait]
pub trait FiatProvider: Send + Sync {
    fn name(&self) -> FiatProviderName;

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>>;

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>>;

    async fn payment_methods(&self) -> Vec<PaymentType> {
        vec![PaymentType::Card, PaymentType::ApplePay, PaymentType::GooglePay]
    }
}

#[async_trait]
impl<T: Send + Sync> FiatProvider for Arc<T>
where
    T: FiatProvider + ?Sized,
{
    fn name(&self) -> FiatProviderName {
        (**self).name()
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_assets().await
    }
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_countries().await
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn payment_methods(&self) -> Vec<PaymentType> {
        (**self).payment_methods().await
    }
}

#[cfg(test)]
mod tests {
    use super::generate_quote_id;
    use uuid::Uuid;

    #[test]
    fn generate_quote_id_returns_uuid() {
        let quote_id = generate_quote_id();
        assert!(Uuid::parse_str(&quote_id).is_ok());
    }
}
