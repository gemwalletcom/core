use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use gem_tracing::{error_with_fields, info_with_fields};
use settings::Settings;
use storage::Database;
use primitives::{FiatTransactionStatus, TransactionId};
use streamer::consumer::MessageConsumer;
use streamer::{FiatWebhook, FiatWebhookPayload, QueueName, StreamProducer, StreamProducerQueue, WalletStreamEvent, WalletStreamPayload};

pub struct FiatWebhookConsumer {
    pub database: Database,
    pub providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    pub stream_producer: StreamProducer,
}

impl FiatWebhookConsumer {
    pub fn new(database: Database, settings: Settings, stream_producer: StreamProducer) -> Self {
        let providers = FiatProviderFactory::new_providers(settings);

        Self {
            database,
            providers,
            stream_producer,
        }
    }
}

#[async_trait]
impl MessageConsumer<FiatWebhookPayload, bool> for FiatWebhookConsumer {
    async fn should_process(&self, _payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        info_with_fields!("received webhook", provider = payload.provider.id(), payload = payload.data.to_string());

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
                    transaction_hash = updated.transaction_hash.as_deref().unwrap_or("")
                );

                if updated.status.0 == FiatTransactionStatus::Complete
                    && let Some(hash) = &updated.transaction_hash
                {
                    let transaction_id = TransactionId::new(updated.asset_id.chain, hash.clone());
                    let _ = self.stream_producer.publish(QueueName::StorePendingTransactions, &transaction_id).await;
                    info_with_fields!("published fiat transaction to pending", provider = provider_id, transaction_id = transaction_id.to_string());
                }

                let payload = WalletStreamPayload {
                    wallet_id: updated.wallet_id,
                    event: WalletStreamEvent::FiatTransaction,
                };
                let _ = self.stream_producer.publish_wallet_stream_events(vec![payload]).await;

                Ok(true)
            }
            Err(e) => {
                error_with_fields!("update_fiat_transaction", &e, provider = provider_id);
                Err(e.into())
            }
        }
    }
}
