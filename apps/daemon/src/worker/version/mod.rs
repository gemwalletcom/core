mod model;
mod version_updater;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use version_updater::VersionClient;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let update_store_versions = run_job("update store versions", Duration::from_secs(43200), {
        let database = database.clone();
        move || {
            let database = database.clone();
            let version_client = VersionClient::new(database);
            async move { version_client.update_store_versions().await }
        }
    });

    vec![Box::pin(update_store_versions)]
}
