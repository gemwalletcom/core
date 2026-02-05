pub mod assets;
pub mod associations;
pub mod blocks;
pub mod consumer_reporter;
pub mod fiat;
pub mod nft;
pub mod notifications;
pub mod prices;
pub mod rewards;
pub mod runner;
pub mod support;
pub mod transactions;

use std::error::Error;
use std::sync::Arc;

use settings::Settings;
use settings_chain::ChainProviders;
use storage::Database;
use streamer::{
    ConsumerConfig, ConsumerStatusReporter, InAppNotificationPayload, QueueName, ShutdownReceiver, StreamProducer, StreamProducerConfig, StreamReader, StreamReaderConfig,
    run_consumer,
};

pub use assets::{run_consumer_fetch_assets, run_consumer_store_assets_associations};
pub use associations::{run_consumer_fetch_coin_associations, run_consumer_fetch_nft_associations, run_consumer_fetch_token_associations};
pub use blocks::run_consumer_fetch_blocks;
pub use fiat::run_consumer_fiat;
pub use prices::{run_consumer_fetch_prices, run_consumer_store_charts, run_consumer_store_prices};
pub use rewards::{run_consumer_rewards, run_rewards_redemption_consumer};
pub use transactions::{run_consumer_fetch_address_transactions, run_consumer_store_transactions};

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

pub async fn run_consumer_in_app_notifications(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::NotificationsInApp;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let consumer = notifications::InAppNotificationsConsumer::new(database, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<InAppNotificationPayload, notifications::InAppNotificationsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter)
        .await
}
