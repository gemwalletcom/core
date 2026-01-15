mod model;
mod version_updater;

use job_runner::run_job;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use storage::ConfigCacher;
use version_updater::VersionClient;

pub async fn jobs(settings: Settings) -> Result<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let update_store_versions = run_job("update store versions", config.get_duration(ConfigKey::VersionTimerUpdateStoreVersions)?, {
        let database = database.clone();
        move || {
            let database = database.clone();
            let version_client = VersionClient::new(database);
            async move { version_client.update_store_versions().await }
        }
    });

    Ok(vec![Box::pin(update_store_versions)])
}
