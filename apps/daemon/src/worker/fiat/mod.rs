use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::{ShutdownReceiver, run_job};
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use tokio::task::JoinHandle;

mod fiat_assets_updater;
pub mod fiat_webhook_consumer;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let update_fiat_assets_job = tokio::spawn(run_job("Update fiat assets", config.get_duration(ConfigKey::FiatTimerUpdateAssets)?, shutdown_rx.clone(), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_fiat_assets("fiat_update_assets").await }
        }
    }));

    let update_fiat_provider_countries_job = tokio::spawn(run_job("Update providers countries", config.get_duration(ConfigKey::FiatTimerUpdateProviderCountries)?, shutdown_rx.clone(), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_fiat_countries("fiat_update_countries").await }
        }
    }));

    let update_fiat_buyable_assets_job = tokio::spawn(run_job("Update fiat buyable/sellable assets", config.get_duration(ConfigKey::FiatTimerUpdateBuyableAssets)?, shutdown_rx.clone(), {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_buyable_sellable_assets().await }
        }
    }));

    let update_trending_fiat_assets_job = tokio::spawn(run_job("Update trending fiat assets", config.get_duration(ConfigKey::FiatTimerUpdateTrending)?, shutdown_rx, {
        let settings = Arc::new(settings.clone());
        let database = database.clone();
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
            async move { fiat_assets_updater.update_trending_fiat_assets().await }
        }
    }));

    Ok(vec![
        update_fiat_assets_job,
        update_fiat_provider_countries_job,
        update_fiat_buyable_assets_job,
        update_trending_fiat_assets_job,
    ])
}
