//! Notifications module for handling push notifications.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use api_connector::PusherClient;
use job_runner::run_job;
use settings::Settings;
use streamer::{run_consumer, ConsumerConfig, NotificationsPayload, QueueName, StreamReader};

mod notifications_consumer;

pub use notifications_consumer::NotificationsConsumer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    vec![
        create_notification_job(
            settings.clone(),
            "Price Alerts Notifications Consumer",
            "price alerts notifications",
            QueueName::NotificationsPriceAlerts,
        ),
        create_notification_job(
            settings.clone(),
            "Transactions Notifications Consumer",
            "transactions notifications",
            QueueName::NotificationsTransactions,
        ),
    ]
}

fn create_notification_job(settings: Settings, name: &'static str, log_prefix: &'static str, queue: QueueName) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    let settings = Arc::new(settings);
    let queue = queue.clone();
    let job_fn = move || {
        let settings = settings.clone();
        let log_prefix = log_prefix;
        let queue = queue.clone();

        async move {
            let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
            let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
            let consumer = NotificationsConsumer::new(pusher_client);

            run_consumer::<NotificationsPayload, NotificationsConsumer, usize>(log_prefix, stream_reader, queue, consumer, ConsumerConfig::default()).await
        }
    };

    Box::pin(run_job(name, Duration::from_secs(u64::MAX), job_fn))
}
