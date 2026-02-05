mod assets_index_updater;
mod nfts_index_updater;
mod perpetuals_index_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use assets_index_updater::AssetsIndexUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use nfts_index_updater::NftsIndexUpdater;
use perpetuals_index_updater::PerpetualsIndexUpdater;
use search_index::SearchIndexClient;
use std::error::Error;
use storage::ConfigCacher;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    let config = ConfigCacher::new(database.clone());

    JobPlanBuilder::with_config(WorkerService::Search, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateAssetsIndex, {
            let database = database.clone();
            let search_index_client = search_index_client.clone();
            move || {
                let updater = AssetsIndexUpdater::new(database.clone(), &search_index_client);
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdatePerpetualsIndex, {
            let database = database.clone();
            let search_index_client = search_index_client.clone();
            move || {
                let updater = PerpetualsIndexUpdater::new(database.clone(), &search_index_client);
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdateNftsIndex, {
            let database = database.clone();
            let search_index_client = search_index_client.clone();
            move || {
                let updater = NftsIndexUpdater::new(database.clone(), &search_index_client);
                async move { updater.update().await }
            }
        })
        .finish()
}
