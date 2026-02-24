mod in_transit_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use in_transit_updater::{InTransitConfig, InTransitUpdater};
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::ConfigKey;
use settings_chain::ProviderFactory;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use swapper::NativeProvider;
use swapper::swapper::GemSwapper;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());

    let in_transit_config = InTransitConfig {
        timeout: config.get_duration(ConfigKey::TransactionInTransitTimeout)?,
        query_limit: config.get_i64(ConfigKey::TransactionInTransitQueryLimit)?,
    };

    let endpoints = ProviderFactory::get_chain_endpoints(&settings);
    let swapper = Arc::new(GemSwapper::new(Arc::new(NativeProvider::new_with_endpoints(endpoints))));

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "transactions_worker").await?;

    JobPlanBuilder::with_config(WorkerService::Transactions, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateInTransitTransactions, {
            let database = database.clone();
            let swapper = swapper.clone();
            let stream_producer = stream_producer.clone();
            move || {
                let updater = InTransitUpdater::new(database.clone(), in_transit_config, swapper.clone(), stream_producer.clone());
                async move { updater.update().await }
            }
        })
        .finish()
}
