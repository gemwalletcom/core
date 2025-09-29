pub mod model;
pub mod support_client;
pub mod support_webhook_consumer;

use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use streamer::{ConsumerConfig, QueueName, StreamReader, StreamReaderConfig, run_consumer};
use support_webhook_consumer::SupportWebhookConsumer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let support_webhook_consumer_job = job_runner::run_job("Support webhook consumer", Duration::from_secs(86000), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings_clone = settings.clone();
            async move {
                let consumer = SupportWebhookConsumer::new(&settings_clone).await.unwrap();
                let config = StreamReaderConfig::new(
                    settings_clone.rabbitmq.url.clone(),
                    "daemon_support_consumer".to_string(),
                    settings_clone.rabbitmq.prefetch,
                );
                let stream_reader = StreamReader::new(config).await.unwrap();
                let _ = run_consumer(
                    "support_webhook_consumer",
                    stream_reader,
                    QueueName::SupportWebhooks,
                    consumer,
                    ConsumerConfig::default(),
                )
                .await;
            }
        }
    });

    vec![Box::pin(support_webhook_consumer_job)]
}
