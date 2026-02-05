pub mod prices_dex_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::{JobVariant, WorkerJob};
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use prices_dex::PriceFeedProvider;
pub use prices_dex_updater::PricesDexUpdater;
use std::time::Duration;

struct ProviderConfig {
    provider_type: PriceFeedProvider,
    name: &'static str,
    url: String,
    timer: u64,
}

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn std::error::Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
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

    providers
        .into_iter()
        .fold(JobPlanBuilder::new(WorkerService::PricesDex, runtime.plan(shutdown_rx)), |builder, provider| {
            let slug = provider.name.to_lowercase();
            let builder = builder.job(JobVariant::labeled(WorkerJob::UpdateDexFeeds, slug.clone()).every(Duration::from_secs(3600)), {
                let url = provider.url.clone();
                let database = database.clone();
                let provider_type = provider.provider_type.clone();
                move || {
                    let url = url.clone();
                    let database = database.clone();
                    let provider_type = provider_type.clone();
                    async move { PricesDexUpdater::new(provider_type, &url, database).update_feeds().await }
                }
            });

            builder.job(JobVariant::labeled(WorkerJob::UpdateDexPrices, slug).every(Duration::from_secs(provider.timer)), {
                let url = provider.url.clone();
                let database = database.clone();
                let provider_type = provider.provider_type.clone();
                move || {
                    let url = url.clone();
                    let database = database.clone();
                    let provider_type = provider_type.clone();
                    async move { PricesDexUpdater::new(provider_type, &url, database).update_prices().await }
                }
            })
        })
        .finish()
}
