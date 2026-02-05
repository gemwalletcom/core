use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::FiatProviderName;
use std::error::Error;
use storage::ConfigCacher;

mod fiat_assets_updater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let provider_names: Vec<FiatProviderName> = FiatProviderFactory::new_providers((*settings).clone())
        .into_iter()
        .map(|provider| provider.name())
        .collect();

    JobPlanBuilder::with_config(WorkerService::Fiat, runtime.plan(shutdown_rx), &config)
        .jobs(WorkerJob::UpdateFiatAssets, provider_names.clone(), |provider_name, _| {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let database = database.clone();
                let provider_name = provider_name.clone();
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone());
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_assets_for(provider_name).await
                }
            }
        })
        .jobs(WorkerJob::UpdateFiatProviderCountries, provider_names.clone(), |provider_name, _| {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let database = database.clone();
                let provider_name = provider_name.clone();
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone());
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_countries_for(provider_name).await
                }
            }
        })
        .job(WorkerJob::UpdateFiatBuyableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_buyable_assets().await }
            }
        })
        .job(WorkerJob::UpdateFiatSellableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_sellable_assets().await }
            }
        })
        .job(WorkerJob::UpdateTrendingFiatAssets, {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let providers = FiatProviderFactory::new_providers((*settings).clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_trending_fiat_assets().await }
            }
        })
        .finish()
}
