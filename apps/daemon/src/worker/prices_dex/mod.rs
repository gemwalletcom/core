pub mod prices_dex_updater;

use job_runner::{ShutdownReceiver, run_job};
use prices_dex::PriceFeedProvider;
pub use prices_dex_updater::PricesDexUpdater;
use settings::Settings;
use std::time::Duration;
use tokio::task::JoinHandle;

struct ProviderConfig {
    provider_type: PriceFeedProvider,
    name: &'static str,
    url: String,
    timer: u64,
}

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Vec<JoinHandle<()>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let providers = vec![
        ProviderConfig {
            provider_type: PriceFeedProvider::Pyth,
            name: "Pyth",
            url: settings.prices.pyth.url.clone(),
            timer: settings.prices.pyth.timer,
        },
        ProviderConfig {
            provider_type: PriceFeedProvider::Jupiter,
            name: "Jupiter",
            url: settings.prices.jupiter.url.clone(),
            timer: settings.prices.jupiter.timer,
        },
    ];

    let mut all_jobs = Vec::new();

    for provider_config in providers {
        let feeds_job_name = format!("Update {} feeds", provider_config.name).leak() as &'static str;
        let feeds_job = tokio::spawn(run_job(feeds_job_name, Duration::from_secs(3600), shutdown_rx.clone(), {
            let url = provider_config.url.clone();
            let database = database.clone();
            let provider_type = provider_config.provider_type.clone();
            move || {
                let url = url.clone();
                let database = database.clone();
                let provider_type = provider_type.clone();
                async move { PricesDexUpdater::new(provider_type, &url, database).update_feeds().await }
            }
        }));

        let prices_job_name = format!("Update {} prices", provider_config.name).leak() as &'static str;
        let prices_job = tokio::spawn(run_job(prices_job_name, Duration::from_secs(provider_config.timer), shutdown_rx.clone(), {
            let url = provider_config.url.clone();
            let database = database.clone();
            let provider_type = provider_config.provider_type.clone();
            move || {
                let url = url.clone();
                let database = database.clone();
                let provider_type = provider_type.clone();
                async move { PricesDexUpdater::new(provider_type, &url, database).update_prices().await }
            }
        }));

        all_jobs.push(feeds_job);
        all_jobs.push(prices_job);
    }

    all_jobs
}
