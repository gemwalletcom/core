mod price_alerts_sender;

use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use price_alerts_sender::PriceAlertSender;
use pricer::PriceAlertClient;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let alerter_interval = config.get_duration(ConfigKey::AlerterInterval)?;

    let price_alerts_job = tokio::spawn(run_job("send_price_alerts", alerter_interval, reporter.clone(), shutdown_rx, {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let settings = Arc::clone(&settings);
            let database = database.clone();

            async move {
                let price_alert_client = PriceAlertClient::new(database.clone());
                let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
                let stream_producer = StreamProducer::new(&rabbitmq_config, "send_price_alerts").await.unwrap();

                PriceAlertSender::new(database, price_alert_client, stream_producer).run_observer().await
            }
        }
    }));

    Ok(vec![price_alerts_job])
}
