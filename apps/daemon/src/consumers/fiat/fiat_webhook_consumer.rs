use std::error::Error;

use async_trait::async_trait;
use fiat::FiatProvider;
use fiat::FiatProviderFactory;
use gem_tracing::{error_with_fields, info_with_fields};
use localizer::LanguageLocalizer;
use primitives::{Device, FiatTransactionStatus, GorushNotification, PushNotification, TransactionId};
use settings::Settings;
use storage::models::FiatTransactionRow;
use storage::{AssetsRepository, Database, WalletsRepository};
use streamer::consumer::MessageConsumer;
use streamer::{FiatWebhook, FiatWebhookPayload, NotificationsPayload, QueueName, StreamProducer, StreamProducerQueue, WalletStreamEvent, WalletStreamPayload};

use crate::pusher::Pusher;

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

    async fn send_fiat_notification(&self, updated: &FiatTransactionRow) -> Result<(), Box<dyn Error + Send + Sync>> {
        let asset = self.database.assets()?.get_asset(&updated.asset_id.0.to_string())?;
        let devices: Vec<Device> = self
            .database
            .wallets()?
            .get_devices_by_wallet_id(updated.wallet_id)?
            .into_iter()
            .map(|d| d.as_primitive())
            .collect();

        let Some(crypto_value) = updated.value.as_deref() else {
            return Ok(());
        };
        let provider = updated.provider_id.0;
        let quote_type = updated.transaction_type.0.clone();
        let notifications: Vec<GorushNotification> = devices
            .iter()
            .filter_map(|device| {
                let localizer = LanguageLocalizer::new_with_language(device.locale.as_str());
                let message = Pusher::fiat_transaction_message(&localizer, &quote_type, provider.name(), &asset, crypto_value).ok()?;
                let data = PushNotification::new_fiat_transaction(asset.id.clone());
                GorushNotification::from_device(device.clone(), message.title, message.message.unwrap_or_default(), data)
            })
            .collect();

        self.stream_producer.publish_notifications_fiat_purchase(NotificationsPayload::new(notifications)).await?;
        Ok(())
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

        let transaction_update = match &payload.payload {
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

        let existing = self.database.fiat()?.get_fiat_transaction(provider_name, &transaction_update.transaction_id)?;
        let updated = self.database.fiat()?.update_fiat_transaction(provider_name, transaction_update)?;

        info_with_fields!(
            "processed webhook",
            provider = provider_id,
            provider_transaction_id = updated.provider_transaction_id.as_deref().unwrap_or(""),
            status = format!("{:?}", updated.status.0),
            quote_id = updated.quote_id.as_str(),
            transaction_hash = updated.transaction_hash.as_deref().unwrap_or("")
        );

        if updated.status.0 == FiatTransactionStatus::Complete && !existing.is_some_and(|row| row.status.0 == FiatTransactionStatus::Complete) {
            if let Some(hash) = &updated.transaction_hash {
                let transaction_id = TransactionId::new(updated.asset_id.0.chain, hash.clone());
                let _ = self.stream_producer.publish(QueueName::StorePendingTransactions, &transaction_id).await;
                info_with_fields!("published fiat transaction to pending", provider = provider_id, transaction_id = transaction_id.to_string());
            }

            if let Err(e) = self.send_fiat_notification(&updated).await {
                error_with_fields!("send_fiat_notification", &*e, provider = provider_id);
            }
        }

        let _ = self
            .stream_producer
            .publish_wallet_stream_events(vec![WalletStreamPayload {
                wallet_id: updated.wallet_id,
                event: WalletStreamEvent::FiatTransaction,
            }])
            .await;

        Ok(true)
    }
}
