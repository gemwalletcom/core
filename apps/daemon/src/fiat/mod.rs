use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
mod fiat_assets_updater;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let update_fiat_assets_job = run_job("Update fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_fiat_assets().await }
        }
    });

    vec![Box::pin(update_fiat_assets_job)]
}
