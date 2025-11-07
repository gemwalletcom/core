mod notifications_consumer;
mod notifications_failed_consumer;

pub use notifications_consumer::NotificationsConsumer;
pub use notifications_failed_consumer::NotificationsFailedConsumer;

use api_connector::PusherClient;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use streamer::{ConsumerConfig, NotificationsFailedPayload, NotificationsPayload, QueueName, StreamProducer, StreamReader, StreamReaderConfig, run_consumer};

pub async fn run(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(settings);

    futures::future::try_join_all(vec![
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsPriceAlerts)),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsTransactions)),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsObservers)),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsSupport)),
        tokio::spawn(run_notifications_failed_consumer(settings.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_notification_consumer(settings: Arc<Settings>, queue: QueueName) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = NotificationsConsumer::new(pusher_client, stream_producer);

    run_consumer::<NotificationsPayload, NotificationsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}

async fn run_notifications_failed_consumer(settings: Arc<Settings>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = "notifications_failed".to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let database = storage::DatabaseClient::new(&settings.postgres.url);
    let consumer = NotificationsFailedConsumer::new(database);

    run_consumer::<NotificationsFailedPayload, NotificationsFailedConsumer, usize>(
        &name,
        stream_reader,
        QueueName::NotificationsFailed,
        consumer,
        ConsumerConfig::default(),
    )
    .await
}
