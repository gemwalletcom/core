mod model;
mod version_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use std::error::Error;
use storage::ConfigCacher;
use version_updater::VersionClient;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let config = ConfigCacher::new(database.clone());

    JobPlanBuilder::with_config(WorkerService::Version, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateStoreVersions, {
            let database = database.clone();
            move || {
                let database = database.clone();
                let version_client = VersionClient::new(database);
                async move { version_client.update_store_versions().await }
            }
        })
        .finish()
}
