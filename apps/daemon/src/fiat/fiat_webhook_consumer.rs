use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use settings::Settings;
use storage::DatabaseClient;
use streamer::consumer::MessageConsumer;
use streamer::{FiatWebhook, FiatWebhookPayload};

pub struct FiatWebhookConsumer {
    pub database: DatabaseClient,
    pub providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatWebhookConsumer {
    pub fn new(database_url: &str, settings: Settings) -> Self {
        let database = DatabaseClient::new(database_url);
        let providers = FiatProviderFactory::new_providers(settings);

        Self { database, providers }
    }
}

#[async_trait]
impl MessageConsumer<FiatWebhookPayload, bool> for FiatWebhookConsumer {
    async fn should_process(&mut self, _payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&mut self, payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for provider in &self.providers {
            if provider.name() == payload.provider {
                let transaction = match &payload.payload {
                    FiatWebhook::OrderId(order_id) => {
                        println!("Fetching order status for provider: {}, order_id: {}", provider.name().id(), order_id);
                        provider.get_order_status(order_id).await?
                    }
                    FiatWebhook::Transaction(transaction) => transaction.clone(),
                };

                println!(
                    "Processing webhook for provider: {}, order_id: {}, symbol: {}, fiat_amount: {} {} status: {:?}",
                    provider.name().id(),
                    transaction.provider_transaction_id,
                    transaction.symbol,
                    transaction.fiat_amount,
                    transaction.fiat_currency,
                    transaction.status
                );

                self.database.fiat().add_fiat_transaction(transaction)?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}
