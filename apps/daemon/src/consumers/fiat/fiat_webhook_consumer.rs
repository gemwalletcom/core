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
        let provider = match self.providers.iter().find(|provider| provider.name() == payload.provider) {
            Some(provider) => provider,
            None => {
                info_with_fields!("ignoring webhook for unsupported provider", provider = payload.provider.id());
                return Ok(false);
            }
        };
        let provider_name = provider.name();
        let provider_id = provider_name.id();

        let transaction = match &payload.payload {
            FiatWebhook::OrderId(order_id) => {
                info_with_fields!("fetching order status", provider = provider_id, provider_transaction_id = order_id);
                match provider.get_order_status(order_id).await {
                    Ok(transaction) => transaction,
                    Err(e) => {
                        error_with_fields!("get_order_status", &*e, provider = provider_id, provider_transaction_id = order_id);
                        return Err(e);
                    }
                }
            }
            FiatWebhook::Transaction(transaction) => transaction.clone(),
            FiatWebhook::None => {
                info_with_fields!("ignoring webhook", provider = provider_id);
                return Ok(true);
            }
        };

        match self.database.fiat()?.update_fiat_transaction(provider_name, transaction) {
            Ok(updated) => {
                info_with_fields!(
                    "processed webhook",
                    provider = provider_id,
                    provider_transaction_id = updated.provider_transaction_id.as_deref().unwrap_or(""),
                    status = format!("{:?}", updated.status.0),
                    quote_id = updated.quote_id.as_str(),
                    address = updated.address.as_deref().unwrap_or(""),
                    transaction_hash = updated.transaction_hash.as_deref().unwrap_or("")
                );
                Ok(true)
            }
            Err(e) => {
                error_with_fields!("update_fiat_transaction", &e, provider = provider_id);
                Err(e.into())
            }
        }
    }
}
