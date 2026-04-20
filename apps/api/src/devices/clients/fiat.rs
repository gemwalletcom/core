use std::collections::BTreeSet;
use std::error::Error;

use fiat::FiatClient;
use primitives::{FiatQuote, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransaction, FiatTransactionData};
use storage::{Database, FiatRepository, WalletsRepository};

pub struct FiatQuotesClient {
    database: Database,
    fiat_client: FiatClient,
}

impl FiatQuotesClient {
    pub fn new(database: Database, fiat_client: FiatClient) -> Self {
        Self { database, fiat_client }
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quotes(request).await
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quote(quote_id).await
    }

    pub async fn get_quote_url(
        &self,
        quote_id: &str,
        wallet_id: i32,
        device_id: i32,
        ip_address: &str,
        locale: &str,
    ) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
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

    pub async fn get_order_status(&self, provider: &str, order_id: &str) -> Result<FiatTransaction, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_order_status(provider, order_id).await
    }

    pub fn get_transactions_by_wallet_id(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?;
        let addresses = subscriptions.into_iter().map(|(_, address)| address.address).collect::<BTreeSet<_>>().into_iter().collect();

        let transactions = FiatRepository::get_fiat_transactions_by_addresses(&mut self.database.fiat()?, addresses)?;

        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }
}
