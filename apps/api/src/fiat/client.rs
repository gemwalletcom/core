use std::error::Error;
use std::str::FromStr;

use fiat::FiatClient;
use primitives::currency::Currency;
use primitives::{Device, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuotes, FiatQuotesData, FiatQuotesDataRequest};
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

    pub async fn get_quotes(
        &self,
        asset_id: &str,
        fiat_amount: Option<f64>,
        crypto_value: Option<&str>,
        quote_type: FiatQuoteType,
        currency: &str,
        wallet_address: &str,
        ip_address: &str,
        provider_id: Option<&str>,
    ) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        if fiat_amount.is_none() && crypto_value.is_none() {
            return Err("Either fiat_amount or crypto_value is required".into());
        }

        let request = FiatQuoteRequest {
            asset_id: asset_id.to_string(),
            quote_type,
            ip_address: ip_address.to_string(),
            fiat_amount,
            fiat_currency: Currency::from_str(currency).unwrap_or(Currency::USD),
            crypto_value: crypto_value.map(|x| x.to_string()),
            wallet_address: wallet_address.to_string(),
            provider_id: provider_id.map(|x| x.to_string()),
        };

        self.fiat_client.get_quotes(request).await
    }

    pub async fn get_quotes_data(
        &self,
        asset_id: &str,
        fiat_amount: f64,
        _quote_type: FiatQuoteType,
        currency: &str,
        ip_address: &str,
        provider_id: Option<&str>,
    ) -> Result<FiatQuotesData, Box<dyn Error + Send + Sync>> {
        let request = FiatQuotesDataRequest {
            fiat_amount,
            fiat_currency: Currency::from_str(currency).unwrap_or(Currency::USD).as_ref().to_string(),
            provider_id: provider_id.map(|x| x.to_string()),
        };

        self.fiat_client.get_quotes_data(request, asset_id, ip_address).await
    }

    pub async fn get_quote_url(&self, quote_id: &str, wallet_address: &str, ip_address: &str, device_id: &str) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        self.get_device(device_id)?;
        self.fiat_client.get_quote_url(quote_id, wallet_address, ip_address, device_id).await
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
