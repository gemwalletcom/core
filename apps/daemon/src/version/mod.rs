mod model;
mod version_updater;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use version_updater::VersionClient;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let _update_appstore_version = run_job("update app store version", Duration::from_secs(43200), {
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

    let update_samsung_store_version = run_job("update samsung store version", Duration::from_secs(43200), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut version_client = VersionClient::new(&settings.postgres.url);
            async move { version_client.update_samsung_store_version().await }
        }
    });

    vec![
        // Box::pin(update_appstore_version),
        Box::pin(update_apk_version),
        Box::pin(update_samsung_store_version),
    ]
}
