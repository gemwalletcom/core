use std::{sync::Arc, time::Duration};
mod price_alerts_sender;
use job_runner::run_job;
use price_alerts_sender::PriceAlertSender;

use api_connector::PusherClient;
use price_alert::PriceAlertClient;
use settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("alerter init");

    let settings = Settings::new().unwrap();

    let price_alerts_job = run_job("Price Alerts", Duration::from_secs(settings.alerter.update_interval_seconds), {
        let settings = Arc::new(settings.clone()); // Clone the Arc to move into the job
        move || {
            let settings = Arc::clone(&settings);

            async move {
                let price_alert_client = PriceAlertClient::new(&settings.postgres.url).await;
                let pusher_client = PusherClient::new(settings.pusher.url.clone());
                let topic = settings.pusher.ios.topic.clone();
                let rules = settings.alerter.rules.clone();

                let mut price_alerts_sender = PriceAlertSender::new(price_alert_client, pusher_client, rules, topic);

                let _ = price_alerts_sender.run().await; // Run the price alert sender
            }
        }
    });

    let _ = tokio::join!(price_alerts_job);

    Ok(())
}
