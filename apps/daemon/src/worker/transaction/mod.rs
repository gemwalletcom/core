mod transaction_updater;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use transaction_updater::TransactionUpdater;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let transaction_updater = run_job("transaction update", Duration::from_secs(86400), {
        let database = database.clone();
        move || {
            let database = database.clone();
            let transaction_updater = TransactionUpdater::new(database);
            async move { transaction_updater.update().await }
        }
    });

    vec![Box::pin(transaction_updater)]
}
