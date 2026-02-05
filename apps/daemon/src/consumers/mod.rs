pub mod consumer_reporter;
pub mod fiat;
pub mod indexer;
pub mod notifications;
pub mod prices;
pub mod rewards;
pub mod runner;
pub mod store;
pub mod support;

use std::error::Error;

use settings::Settings;
use settings_chain::ChainProviders;
use streamer::{ConsumerConfig, QueueName, StreamProducer, StreamProducerConfig, StreamReader, StreamReaderConfig};

pub use fiat::run_consumer_fiat;
pub use indexer::run_consumer_indexer;
pub use prices::run_consumer_fetch_prices;
pub use rewards::run_consumer_rewards;
pub use store::run_consumer_store;
pub use support::run_consumer_support;

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
