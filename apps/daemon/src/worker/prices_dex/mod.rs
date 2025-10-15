pub mod prices_dex_updater;

use job_runner::run_job;
use prices_dex::PriceFeedProvider;
pub use prices_dex_updater::PricesDexUpdater;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

struct ProviderConfig {
    provider_type: PriceFeedProvider,
    name: &'static str,
    url: String,
    timer: u64,
}

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
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
        let feeds_job = run_job(feeds_job_name, Duration::from_secs(3600), {
            let url = provider_config.url.clone();
            let database_url = settings.postgres.url.clone();
            let provider_type = provider_config.provider_type.clone();
            move || {
                let url = url.clone();
                let database_url = database_url.clone();
                let provider_type = provider_type.clone();
                async move { PricesDexUpdater::new(provider_type, &url, &database_url).update_feeds().await }
            }
        });

        let prices_job_name = format!("Update {} prices", provider_config.name).leak() as &'static str;
        let prices_job = run_job(prices_job_name, Duration::from_secs(provider_config.timer), {
            let url = provider_config.url.clone();
            let database_url = settings.postgres.url.clone();
            let provider_type = provider_config.provider_type.clone();
            move || {
                let url = url.clone();
                let database_url = database_url.clone();
                let provider_type = provider_type.clone();
                async move { PricesDexUpdater::new(provider_type, &url, &database_url).update_prices().await }
            }
        });

        all_jobs.push(Box::pin(feeds_job) as Pin<Box<dyn Future<Output = ()> + Send>>);
        all_jobs.push(Box::pin(prices_job) as Pin<Box<dyn Future<Output = ()> + Send>>);
    }

    all_jobs
}
