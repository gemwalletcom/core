mod price_alerts_sender;

use job_runner::run_job;
use price_alerts_sender::PriceAlertSender;
use pricer::PriceAlertClient;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use streamer::StreamProducer;

pub async fn jobs(settings: Settings) -> Result<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);

    let alerter_interval = database.client()?.config().get_config_duration(ConfigKey::AlerterInterval)?;

    let price_alerts_job = run_job("Price Alerts", alerter_interval, {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let settings = Arc::clone(&settings);
            let database = database.clone();

            async move {
                let price_alert_client = PriceAlertClient::new(database.clone());
                let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "price_alerts").await.unwrap();

                PriceAlertSender::new(database, price_alert_client, stream_producer).run_observer().await
            }
        }
    });

    Ok(vec![Box::pin(price_alerts_job)])
}
