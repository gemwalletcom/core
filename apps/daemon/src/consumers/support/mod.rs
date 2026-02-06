pub mod support_webhook_consumer;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use streamer::{ConsumerStatusReporter, QueueName, ShutdownReceiver, SupportWebhookPayload, run_consumer};

use crate::consumers::{consumer_config, reader_for_queue};

use support_webhook_consumer::SupportWebhookConsumer;

pub async fn run_consumer_support(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::SupportWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}
