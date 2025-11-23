use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use gem_tracing::{error_with_fields, info_with_fields};
use settings::Settings;
use storage::Database;
use streamer::consumer::MessageConsumer;
use streamer::{FiatWebhook, FiatWebhookPayload};

pub struct FiatWebhookConsumer {
    pub database: Database,
    pub providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatWebhookConsumer {
    pub fn new(database: Database, settings: Settings) -> Self {
        let providers = FiatProviderFactory::new_providers(settings);

        Self { database, providers }
    }
}

#[async_trait]
impl MessageConsumer<FiatWebhookPayload, bool> for FiatWebhookConsumer {
    async fn should_process(&self, _payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for provider in &self.providers {
            if provider.name() == payload.provider {
                let transaction = match &payload.payload {
                    FiatWebhook::OrderId(order_id) => {
                        info_with_fields!("fetching order status", provider = provider.name().id(), order_id = order_id);
                        match provider.get_order_status(order_id).await {
                            Ok(transaction) => transaction,
                            Err(e) => {
                                error_with_fields!("get_order_status", &*e, provider = provider.name().id(), order_id = order_id);
                                return Err(e);
                            }
                        }
                    }
                    FiatWebhook::Transaction(transaction) => transaction.clone(),
                    FiatWebhook::None => {
                        info_with_fields!("ignoring webhook", provider = provider.name().id());
                        return Ok(true);
                    }
                };

                info_with_fields!(
                    "processing webhook",
                    provider = provider.name().id(),
                    order_id = transaction.provider_transaction_id.as_str(),
                    symbol = transaction.symbol.as_str(),
                    fiat_amount = transaction.fiat_amount.to_string(),
                    fiat_currency = transaction.fiat_currency.as_str(),
                    status = format!("{:?}", transaction.status)
                );

                match self.database.client()?.fiat().add_fiat_transaction(transaction) {
                    Ok(_) => return Ok(true),
                    Err(e) => {
                        error_with_fields!("add_fiat_transaction", &e, provider = provider.name().id());
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(false)
    }
}
