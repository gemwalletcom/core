mod transaction_updater;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use transaction_updater::TransactionUpdater;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let transaction_updater = run_job("transaction update", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut transaction_updater = TransactionUpdater::new(&settings.postgres.url);
            async move { transaction_updater.update().await }
        }
    });

    vec![Box::pin(transaction_updater)]
}
