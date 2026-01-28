mod transaction_updater;

use job_runner::{ShutdownReceiver, run_job};
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use storage::ConfigCacher;
use tokio::task::JoinHandle;
use transaction_updater::TransactionUpdater;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let transaction_updater = tokio::spawn(run_job("transaction update", config.get_duration(ConfigKey::TransactionTimerUpdater)?, shutdown_rx, {
        let database = database.clone();
        move || {
            let database = database.clone();
            let transaction_updater = TransactionUpdater::new(database);
            async move { transaction_updater.update().await }
        }
    }));

    Ok(vec![transaction_updater])
}
