mod transaction_updater;

use job_runner::run_job;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use storage::ConfigCacher;
use transaction_updater::TransactionUpdater;

pub async fn jobs(settings: Settings) -> Result<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let transaction_updater = run_job("transaction update", config.get_duration(ConfigKey::TransactionTimerUpdater)?, {
        let database = database.clone();
        move || {
            let database = database.clone();
            let transaction_updater = TransactionUpdater::new(database);
            async move { transaction_updater.update().await }
        }
    });

    Ok(vec![Box::pin(transaction_updater)])
}
