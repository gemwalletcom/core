mod model;
mod version_updater;

use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use tokio::task::JoinHandle;
use version_updater::VersionClient;

pub async fn jobs(settings: Settings, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let update_store_versions = tokio::spawn(run_job(
        "update store versions",
        config.get_duration(ConfigKey::VersionTimerUpdateStoreVersions)?,
        reporter.clone(),
        shutdown_rx,
        {
            let database = database.clone();
            move || {
                let database = database.clone();
                let version_client = VersionClient::new(database);
                async move { version_client.update_store_versions().await }
            }
        },
    ));

    Ok(vec![update_store_versions])
}
