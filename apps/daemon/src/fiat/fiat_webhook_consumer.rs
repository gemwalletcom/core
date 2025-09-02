use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use settings::Settings;
use storage::DatabaseClient;
use streamer::consumer::MessageConsumer;
use streamer::FiatWebhookPayload;

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
                let order_id = match provider.webhook_order_id(payload.data.clone()).await {
                    Ok(order_id) => order_id,
                    Err(e) => {
                        println!(
                            "Failed to get order ID for webhook for provider {} with data: \n\n {:?}\n\n failed: {}",
                            provider.name().id(),
                            payload.data,
                            e
                        );
                        return Ok(false);
                    }
                };

                let transaction = provider.get_order_status(&order_id).await?;

                println!(
                    "Processing webhook for provider: {}, order_id: {}, symbol: {}, fiat_amount: {} {} status: {:?}",
                    provider.name().id(),
                    order_id,
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
