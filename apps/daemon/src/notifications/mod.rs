//! Notifications module for handling push notifications.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use api_connector::PusherClient;
use job_runner::run_job;
use settings::Settings;
use streamer::{QueueName, StreamReader};

mod notifications_consumer;

pub use notifications_consumer::NotificationsConsumer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let price_alerts_notifications = run_job("Price Alerts Notifications Consumer", Duration::from_secs(u64::MAX), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = settings.clone();
            async move {
                let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
                let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
                NotificationsConsumer::new(pusher_client, stream_reader)
                    .run("price alerts notifications consumer", QueueName::NotificationsPriceAlerts)
                    .await
            }
        }
    });

    let transactions_notifications = run_job("Transactions Notifications Consumer", Duration::from_secs(u64::MAX), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = settings.clone();
            async move {
                let stream_reader = StreamReader::new(&settings.rabbitmq.url).await.unwrap();
                let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
                NotificationsConsumer::new(pusher_client, stream_reader)
                    .run("transactions notifications consumer", QueueName::NotificationsTransactions)
                    .await
            }
        }
    });

    vec![Box::pin(price_alerts_notifications), Box::pin(transactions_notifications)]
}
