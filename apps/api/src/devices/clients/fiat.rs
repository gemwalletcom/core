use std::error::Error;

use fiat::FiatClient;
use primitives::{FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes};

pub struct FiatQuotesClient {
    fiat_client: FiatClient,
}

impl FiatQuotesClient {
    pub fn new(fiat_client: FiatClient) -> Self {
        Self { fiat_client }
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quotes(request).await
    }

    pub async fn get_quote_url(
        &self,
        quote_id: &str,
        wallet_id: i32,
        device_id: i32,
        ip_address: &str,
        locale: &str,
    ) -> Result<(FiatQuoteUrl, FiatQuote), Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quote_url(quote_id, wallet_id, device_id, ip_address, locale).await
    }

    pub async fn get_on_ramp_assets(&self) -> Result<primitives::FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_on_ramp_assets().await
    }

    pub async fn get_off_ramp_assets(&self) -> Result<primitives::FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_off_ramp_assets().await
    }

    pub async fn process_and_publish_webhook(&self, provider: &str, webhook_data: serde_json::Value) -> Result<streamer::FiatWebhookPayload, Box<dyn Error + Send + Sync>> {
        self.fiat_client.process_and_publish_webhook(provider, webhook_data).await
    }

    pub async fn get_order_status(&self, provider: &str, order_id: &str) -> Result<primitives::FiatTransaction, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_order_status(provider, order_id).await
    }
}
