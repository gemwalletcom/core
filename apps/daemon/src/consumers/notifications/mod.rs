mod in_app_notifications_consumer;
mod notifications_consumer;
mod notifications_failed_consumer;

pub use in_app_notifications_consumer::InAppNotificationsConsumer;
pub use notifications_consumer::NotificationsConsumer;
pub use notifications_failed_consumer::NotificationsFailedConsumer;

use api_connector::PusherClient;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::Database;
use streamer::{ConsumerConfig, NotificationsFailedPayload, NotificationsPayload, QueueName, ShutdownReceiver, StreamProducer, StreamReader, StreamReaderConfig, run_consumer};

fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
    }
}

pub async fn run(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let settings = Arc::new(settings);
    let database = Arc::new(database);

    futures::future::try_join_all(vec![
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsPriceAlerts, shutdown_rx.clone())),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsTransactions, shutdown_rx.clone())),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsObservers, shutdown_rx.clone())),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsSupport, shutdown_rx.clone())),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsRewards, shutdown_rx.clone())),
        tokio::spawn(run_notifications_failed_consumer(settings.clone(), database.clone(), QueueName::NotificationsFailed, shutdown_rx.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_notification_consumer(settings: Arc<Settings>, queue: QueueName, shutdown_rx: ShutdownReceiver) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = NotificationsConsumer::new(pusher_client, stream_producer);

    run_consumer::<NotificationsPayload, NotificationsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx).await
}

async fn run_notifications_failed_consumer(settings: Arc<Settings>, database: Arc<Database>, queue: QueueName, shutdown_rx: ShutdownReceiver) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = NotificationsFailedConsumer::new((*database).clone());

    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<NotificationsFailedPayload, NotificationsFailedConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx).await
}
