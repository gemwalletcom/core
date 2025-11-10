use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

mod fiat_assets_updater;
pub mod fiat_webhook_consumer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);

    let update_fiat_assets_job = run_job("Update fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_fiat_assets("fiat_update_assets").await }
        }
    });

    let update_fiat_provider_countries_job = run_job("Update providers countries", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_fiat_countries("fiat_update_countries").await }
        }
    });

    let update_fiat_buyable_assets_job = run_job("Update fiat buyable/sellable assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_buyable_sellable_assets().await }
        }
    });

    let update_trending_fiat_assets_job = run_job("Update trending fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_trending_fiat_assets().await }
        }
    });

    vec![
        Box::pin(update_fiat_assets_job),
        Box::pin(update_fiat_provider_countries_job),
        Box::pin(update_fiat_buyable_assets_job),
        Box::pin(update_trending_fiat_assets_job),
    ]
}
