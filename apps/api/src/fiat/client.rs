use std::error::Error;
use std::str::FromStr;

use fiat::FiatClient;
use primitives::currency::Currency;
use primitives::{Device, FiatQuote, FiatQuoteOldRequest, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlRequest, FiatQuotes, FiatQuotesOld};
use storage::Database;

pub struct FiatQuotesClient {
    fiat_client: FiatClient,
    database: Database,
}

impl FiatQuotesClient {
    pub fn new(fiat_client: FiatClient, database: Database) -> Self {
        Self { fiat_client, database }
    }

    pub fn get_device(&self, device_id: &str) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.devices().get_device(device_id)?)
    }

    pub async fn get_quotes_old(
        &self,
        asset_id: &str,
        fiat_amount: Option<f64>,
        crypto_value: Option<&str>,
        quote_type: FiatQuoteType,
        currency: &str,
        wallet_address: &str,
        ip_address: &str,
        provider_id: Option<&str>,
    ) -> Result<FiatQuotesOld, Box<dyn Error + Send + Sync>> {
        if fiat_amount.is_none() && crypto_value.is_none() {
            return Err("Either fiat_amount or crypto_value is required".into());
        }

        let request = FiatQuoteOldRequest {
            asset_id: asset_id.to_string(),
            quote_type,
            ip_address: ip_address.to_string(),
            fiat_amount,
            fiat_currency: Currency::from_str(currency).unwrap_or(Currency::USD),
            crypto_value: crypto_value.map(|x| x.to_string()),
            wallet_address: wallet_address.to_string(),
            provider_id: provider_id.map(|x| x.to_string()),
        };

        self.fiat_client.get_quotes_old(request).await
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quotes(request).await
    }

    pub async fn get_quote_url(&self, request: &FiatQuoteUrlRequest, ip_address: &str) -> Result<(FiatQuoteUrl, FiatQuote), Box<dyn Error + Send + Sync>> {
        self.get_device(&request.device_id)?;
        self.fiat_client
            .get_quote_url(&request.quote_id, &request.wallet_address, ip_address, &request.device_id)
            .await
    }

    pub async fn get_on_ramp_assets(&self) -> Result<primitives::FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_on_ramp_assets().await
    }

    pub async fn get_off_ramp_assets(&self) -> Result<primitives::FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_off_ramp_assets().await
    }

    pub async fn process_and_publish_webhook(
        &self,
        provider: &str,
        webhook_data: serde_json::Value,
    ) -> Result<streamer::FiatWebhookPayload, Box<dyn Error + Send + Sync>> {
        self.fiat_client.process_and_publish_webhook(provider, webhook_data).await
    }

    pub async fn get_order_status(&self, provider: &str, order_id: &str) -> Result<primitives::FiatTransaction, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_order_status(provider, order_id).await
    }
}
