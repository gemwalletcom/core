mod alerter;
mod assets;
mod device_updater;
mod fiat;
mod pricer;
mod tokenlist_updater;
mod transaction_updater;
mod version_updater;

use crate::device_updater::DeviceUpdater;
use crate::tokenlist_updater::Client as TokenListClient;
use crate::transaction_updater::TransactionUpdater;
use crate::version_updater::Client as VersionClient;
use api_connector::AssetsClient;
use job_runner::run_job;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    println!("daemon init");

    let service = std::env::args().nth(1).unwrap_or_default();

    println!("daemon start service: {service}");

    let settings = settings::Settings::new().unwrap();

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
    let services: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = match service.as_str() {
        "alerter" => alerter::jobs(settings.clone()).await,
        "pricer" => pricer::jobs(settings.clone()).await,
        "fiat" => fiat::jobs(settings.clone()).await,
        "assets" => assets::jobs(settings.clone()).await,
        _ => {
            vec![
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
