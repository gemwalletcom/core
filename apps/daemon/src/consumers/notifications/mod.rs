mod notifications_consumer;

pub use notifications_consumer::NotificationsConsumer;

use api_connector::PusherClient;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use streamer::{ConsumerConfig, NotificationsPayload, QueueName, StreamReader, StreamReaderConfig, run_consumer};

pub async fn run(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(settings);

    futures::future::try_join_all(vec![
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsPriceAlerts)),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsTransactions)),
        tokio::spawn(run_notification_consumer(settings.clone(), QueueName::NotificationsObservers)),
        tokio::spawn(run_notification_consumer(settings, QueueName::NotificationsSupport)),
    ])
    .await?;

    Ok(())
}

async fn run_notification_consumer(settings: Arc<Settings>, queue: QueueName) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let consumer = NotificationsConsumer::new(pusher_client);

    run_consumer::<NotificationsPayload, NotificationsConsumer, usize>(&name, stream_reader, queue, consumer, ConsumerConfig::default()).await
}
