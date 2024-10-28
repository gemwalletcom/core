mod price_alerts_sender;

use job_runner::run_job;
use price_alerts_sender::PriceAlertSender;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use api_connector::PusherClient;
use price_alert::PriceAlertClient;
use settings::Settings;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output=()> + Send>>> {
    let price_alerts_job = run_job("Price Alerts", Duration::from_secs(settings.alerter.update_interval_seconds), {
        let settings = Arc::new(settings.clone());
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
                ).run()
                    .await
            }
        }
    });

    vec![
        Box::pin(price_alerts_job),
    ]
}
