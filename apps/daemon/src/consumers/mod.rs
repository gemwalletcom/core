pub mod consumer_reporter;
pub mod fiat;
pub mod chain;
pub mod notifications;
pub mod prices;
pub mod rewards;
pub mod runner;
pub mod store;
pub mod support;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use settings_chain::ChainProviders;
use streamer::{ConsumerConfig, ConsumerStatusReporter, QueueName, ShutdownReceiver, StreamProducer, StreamProducerConfig, StreamReader, StreamReaderConfig, run_consumer};

pub use fiat::run_consumer_fiat;
pub use chain::run_consumer_chain;
pub use prices::run_consumer_fetch_prices;
pub use rewards::run_consumer_rewards;
pub use store::run_consumer_store;

pub fn chain_providers(settings: &Settings, name: &str) -> ChainProviders {
    ChainProviders::from_settings(settings, &settings::service_user_agent("consumer", Some(name)))
}

pub(crate) fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
    }
}

pub(crate) async fn reader_for_queue(settings: &Settings, queue: &QueueName) -> Result<(String, StreamReader), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let reader = StreamReader::new(config).await?;
    Ok((name, reader))
}

fn producer_config(settings: &Settings) -> StreamProducerConfig {
    StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay)
}

pub(crate) async fn producer_for_queue(settings: &Settings, name: &str) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
    let config = producer_config(settings);
    StreamProducer::new(&config, name).await
}

pub async fn run_consumer_support(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use streamer::SupportWebhookPayload;
    use support::support_webhook_consumer::SupportWebhookConsumer;

    let queue = QueueName::SupportWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}

