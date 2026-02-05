mod transaction_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use std::error::Error;
use storage::ConfigCacher;
use transaction_updater::TransactionUpdater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let config = ConfigCacher::new(database.clone());

    JobPlanBuilder::with_config(WorkerService::Transaction, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::CleanupProcessedTransactions, {
            let database = database.clone();
            move || {
                let database = database.clone();
                let transaction_updater = TransactionUpdater::new(database);
                async move { transaction_updater.update().await }
            }
        })
        .finish()
}
