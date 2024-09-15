use std::{sync::Arc, time::Duration};
mod price_alerts_sender;
use job_runner::run_job;
use price_alerts_sender::PriceAlertSender;

use api_connector::PusherClient;
use price_alert::PriceAlertClient;
use settings::Settings;

#[tokio::main]
async fn main() {
    println!("alerter init");

    let settings = Settings::new().unwrap();

    let price_alerts_job = run_job("Price Alerts", Duration::from_secs(settings.alerter.update_interval_seconds), {
        let settings = Arc::new(settings.clone()); // Clone the Arc to move into the job
        move || {
            let settings = Arc::clone(&settings);

            async move {
                let price_alert_client = PriceAlertClient::new(&settings.postgres.url).await;
                let pusher_client = PusherClient::new(settings.pusher.url.clone());

                PriceAlertSender::new(
                    price_alert_client,
                    pusher_client,
                    settings.alerter.rules.clone(),
                    settings.pusher.ios.topic.clone(),
                )
                .run()
                .await
            }
        }
    });

    let _ = tokio::join!(price_alerts_job);
}
