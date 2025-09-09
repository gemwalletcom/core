use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use gem_tracing::{error_with_context, info_with_context};
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
                        info_with_context("fetching order status", &[("provider", &provider.name().id()), ("order_id", order_id)]);
                        match provider.get_order_status(order_id).await {
                            Ok(transaction) => transaction,
                            Err(e) => {
                                error_with_context("get_order_status", &*e, &[("provider", &provider.name().id()), ("order_id", order_id)]);
                                return Err(e);
                            }
                        }
                    }
                    FiatWebhook::Transaction(transaction) => transaction.clone(),
                    FiatWebhook::None => {
                        info_with_context("ignoring webhook", &[("provider", &provider.name().id())]);
                        return Ok(true);
                    }
                };

                info_with_context(
                    "processing webhook",
                    &[
                        ("provider", &provider.name().id()),
                        ("order_id", &transaction.provider_transaction_id),
                        ("symbol", &transaction.symbol),
                        ("fiat_amount", &transaction.fiat_amount.to_string()),
                        ("fiat_currency", transaction.fiat_currency.as_ref()),
                        ("status", &format!("{:?}", transaction.status)),
                    ],
                );

                match self.database.fiat().add_fiat_transaction(transaction) {
                    Ok(_) => return Ok(true),
                    Err(e) => {
                        error_with_context("add_fiat_transaction", &e, &[("provider", &provider.name().id())]);
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(false)
    }
}
