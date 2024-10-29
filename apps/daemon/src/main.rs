mod device_updater;
mod fiat_assets_updater;
mod tokenlist_updater;
mod version_updater;
mod transaction_updater;
mod alerter;

use crate::device_updater::DeviceUpdater;
use crate::tokenlist_updater::Client as TokenListClient;
use crate::transaction_updater::TransactionUpdater;
use crate::version_updater::Client as VersionClient;
use api_connector::AssetsClient;
use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::run_job;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    println!("daemon init");
    let settings = settings::Settings::new().unwrap();
    let service = std::env::args().nth(1).unwrap_or_default();

    let update_fiat_assets = run_job("update fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_fiat_assets().await }
        }
    });

    let update_appstore_version = run_job("update app store version", Duration::from_secs(43200), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut version_client = VersionClient::new(&settings.postgres.url);
            async move { version_client.update_ios_version().await }
        }
    });

    let update_apk_version = run_job("update apk version", Duration::from_secs(43200), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut version_client = VersionClient::new(&settings.postgres.url);
            async move { version_client.update_apk_version().await }
        }
    });

    let device_updater = run_job("device updater", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut device_updater = DeviceUpdater::new(&settings.postgres.url);
            async move { device_updater.update().await }
        }
    });

    let token_list_updater = run_job("token list update", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let assets_client = AssetsClient::new(&settings.assets.url);
            let mut tokenlist_client = TokenListClient::new(&settings.postgres.url, assets_client);
            async move { tokenlist_client.update().await }
        }
    });

    let transaction_updater = run_job("transaction update", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut transaction_updater = TransactionUpdater::new(&settings.postgres.url);
            async move { transaction_updater.update().await }
        }
    });


    // Pin the futures when creating the services vector
    let services: Vec<Pin<Box<dyn Future<Output=()> + Send>>> = match service.as_str() {
        "alerter" => {
            alerter::jobs(settings.clone()).await
        }
        _ => {
            vec![
                Box::pin(update_fiat_assets),
                Box::pin(update_appstore_version),
                Box::pin(update_apk_version),
                Box::pin(device_updater),
                Box::pin(token_list_updater),
                Box::pin(transaction_updater),
            ]
        }
    };

    let _ = futures::future::join_all(services).await;
}
