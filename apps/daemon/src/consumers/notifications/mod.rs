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
use streamer::{
    ConsumerConfig, ConsumerStatusReporter, InAppNotificationPayload, NotificationsFailedPayload, NotificationsPayload, QueueName, ShutdownReceiver, StreamProducer,
    StreamProducerConfig, StreamReader, run_consumer,
};

use crate::consumers::reader_config;

fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
    }
}

pub async fn run(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let settings = Arc::new(settings);
    let database = Arc::new(database);

    futures::future::try_join_all(vec![
        tokio::spawn(run_notification_consumer(
            settings.clone(),
            QueueName::NotificationsPriceAlerts,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_notification_consumer(
            settings.clone(),
            QueueName::NotificationsTransactions,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_notification_consumer(
            settings.clone(),
            QueueName::NotificationsObservers,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_notification_consumer(
            settings.clone(),
            QueueName::NotificationsSupport,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_notification_consumer(
            settings.clone(),
            QueueName::NotificationsRewards,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_notifications_failed_consumer(
            settings.clone(),
            database.clone(),
            QueueName::NotificationsFailed,
            shutdown_rx.clone(),
            reporter.clone(),
        )),
        tokio::spawn(run_in_app_notifications_consumer(settings.clone(), database.clone(), shutdown_rx.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_notification_consumer(
    settings: Arc<Settings>,
    queue: QueueName,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let stream_reader = StreamReader::new(reader_config(&settings.rabbitmq, name.clone())).await?;
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, &name).await?;
    let consumer = NotificationsConsumer::new(pusher_client, stream_producer);

    run_consumer::<NotificationsPayload, NotificationsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter)
        .await
}

async fn run_notifications_failed_consumer(
    settings: Arc<Settings>,
    database: Arc<Database>,
    queue: QueueName,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let stream_reader = StreamReader::new(reader_config(&settings.rabbitmq, name.clone())).await?;
    let consumer = NotificationsFailedConsumer::new((*database).clone());

    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<NotificationsFailedPayload, NotificationsFailedConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}

async fn run_in_app_notifications_consumer(
    settings: Arc<Settings>,
    database: Arc<Database>,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::NotificationsInApp;
    let name = queue.to_string();
    let stream_reader = StreamReader::new(reader_config(&settings.rabbitmq, name.clone())).await?;
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, &name).await?;
    let consumer = InAppNotificationsConsumer::new((*database).clone(), stream_producer);

    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<InAppNotificationPayload, InAppNotificationsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}
