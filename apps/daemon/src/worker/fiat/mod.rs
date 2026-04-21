use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use fiat_rates_updater::FiatRatesUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use pricer::PriceClient;
use primitives::FiatProviderName;
use std::error::Error;
use storage::ConfigCacher;

mod fiat_assets_updater;
mod fiat_rates_updater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());

    let cacher_client = CacherClient::new(&settings.redis.url).await;

    JobPlanBuilder::with_config(WorkerService::Fiat, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateFiatRates, {
            let settings = settings.clone();
            let database = database.clone();
            let cacher_client = cacher_client.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                let cacher_client = cacher_client.clone();
                async move {
                    let client = CoinGeckoClient::new(&settings.coingecko.key.secret);
                    let price_client = PriceClient::new(database, cacher_client);
                    FiatRatesUpdater::new(client, price_client).update().await
                }
            }
        })
        .jobs(WorkerJob::UpdateFiatAssets, FiatProviderName::all(), |provider, _| {
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                let provider = provider;
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone());
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_assets_for(provider).await
                }
            }
        })
        .jobs(WorkerJob::UpdateFiatProviderCountries, FiatProviderName::all(), |provider, _| {
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                let provider = provider;
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone());
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_countries_for(provider).await
                }
            }
        })
        .job(WorkerJob::UpdateFiatBuyableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_buyable_assets().await }
            }
        })
        .job(WorkerJob::UpdateFiatSellableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_sellable_assets().await }
            }
        })
        .job(WorkerJob::UpdateTrendingFiatAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_trending_fiat_assets().await }
            }
        })
        .finish()
}
