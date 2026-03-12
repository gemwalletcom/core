pub mod support_webhook_consumer;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::{ConsumerStatusReporter, QueueName, ShutdownReceiver, StreamProducer, StreamProducerConfig, SupportWebhookPayload, run_consumer};
use support::{ChatwootApiClient, ClaudeClient, SupportBotClient, SupportClient};

use crate::consumers::{consumer_config, reader_for_queue};

use support_webhook_consumer::SupportWebhookConsumer;

pub async fn run_consumer_support(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "daemon_support_producer", shutdown_rx.clone()).await?;

    let support_client = SupportClient::new(database, stream_producer);

    let chatwoot_client = ChatwootApiClient::new(settings.chatwoot.url.clone(), settings.chatwoot.key.secret.clone());
    let claude_client = ClaudeClient::new(settings.claude.url.clone(), settings.claude.key.secret.clone());
    let bot_client = SupportBotClient::new(chatwoot_client, claude_client, config);

    let consumer = SupportWebhookConsumer::new(support_client, bot_client);

    let queue = QueueName::SupportWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}
