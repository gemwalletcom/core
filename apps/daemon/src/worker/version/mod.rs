mod model;
mod version_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use std::error::Error;
use storage::ConfigCacher;
use version_updater::VersionUpdater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let config = ConfigCacher::new(database.clone());

    JobPlanBuilder::with_config(WorkerService::Version, runtime.plan(shutdown_rx), &config)
        .jobs(WorkerJob::UpdateStoreVersion, VersionUpdater::stores(), |store, _| {
            let store = *store;
            let database = database.clone();
            move || {
                let updater = VersionUpdater::new(database.clone());
                async move { updater.update_store(store).await }
            }
        })
        .finish()
}
