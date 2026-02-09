mod device_updater;
mod model;
mod observers;
mod transaction_updater;
mod version_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use cacher::CacherClient;
use device_updater::DeviceUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use observers::InactiveDevicesObserver;
use std::error::Error;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use transaction_updater::TransactionUpdater;
use version_updater::VersionUpdater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let cacher_client = CacherClient::new(settings.redis.url.as_str()).await;

    JobPlanBuilder::with_config(WorkerService::System, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::CleanupProcessedTransactions, {
            let database = database.clone();
            move || {
                let database = database.clone();
                let transaction_updater = TransactionUpdater::new(database);
                async move { transaction_updater.update().await }
            }
        })
        .job(WorkerJob::CleanupStaleDeviceSubscriptions, {
            let database = database.clone();
            move || {
                let device_updater = DeviceUpdater::new(database.clone());
                async move { device_updater.update().await }
            }
        })
        .job(WorkerJob::ObserveInactiveDevices, {
            let database = database.clone();
            let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
            let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
            let stream_producer = StreamProducer::new(&rabbitmq_config, "observe_inactive_devices").await?;
            move || {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                let cacher_client = cacher_client.clone();
                async move {
                    let observer = InactiveDevicesObserver::new(database, cacher_client, stream_producer);
                    observer.observe().await
                }
            }
        })
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
