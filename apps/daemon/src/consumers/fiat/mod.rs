pub mod fiat_webhook_consumer;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use storage::Database;
use streamer::{ConsumerStatusReporter, FiatWebhookPayload, QueueName, ShutdownReceiver, run_consumer};

use crate::consumers::{consumer_config, producer_for_queue, reader_for_queue};

use fiat_webhook_consumer::FiatWebhookConsumer;

pub async fn run_consumer_fiat(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FiatOrderWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let stream_producer = producer_for_queue(&settings, &format!("{name}_producer"), shutdown_rx.clone()).await?;
    let consumer = FiatWebhookConsumer::new(database, settings.clone(), stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<FiatWebhookPayload, FiatWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}
