mod perpetual_address_refresher;
mod perpetual_classifier;
mod perpetual_observer;

use cacher::CacherClient;
use job_runner::{JobHandle, ShutdownReceiver};
use perpetual_address_refresher::PerpetualAddressRefresher;
use perpetual_classifier::{PerpetualPositionClassifier, PerpetualPriorityConfig};
use perpetual_observer::PerpetualPositionObserver;
use primitives::{Chain, ConfigKey};
use settings_chain::ChainProviders;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "perpetuals_worker", shutdown_rx.clone()).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;

    let providers = Arc::new(ChainProviders::from_settings(
        &settings,
        &settings::service_user_agent("daemon", Some("perpetual_observer")),
    ));
    let priority_config = PerpetualPriorityConfig {
        trigger_bps: config.get_i64(ConfigKey::PerpetualPriorityTriggerBps)?,
        liquidation_bps: config.get_i64(ConfigKey::PerpetualPriorityLiquidationBps)?,
    };
    let refresher = Arc::new(PerpetualAddressRefresher::new(providers.clone(), database.clone(), cacher.clone()));

    ctx.plan_builder(WorkerService::Perpetuals, &config, shutdown_rx)
        .jobs(WorkerJob::ClassifyPerpetualAddresses, Chain::perpetual_chains(), |chain, _| {
            let classifier = Arc::new(PerpetualPositionClassifier::new(chain, providers.clone(), cacher.clone(), priority_config));
            move |_| {
                let classifier = classifier.clone();
                async move { classifier.classify().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualActiveAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(PerpetualPositionObserver::new(chain, providers.clone(), cacher.clone(), stream_producer.clone()));
            move |_| {
                let observer = observer.clone();
                async move { observer.observe_active().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualPriorityAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(PerpetualPositionObserver::new(chain, providers.clone(), cacher.clone(), stream_producer.clone()));
            move |_| {
                let observer = observer.clone();
                async move { observer.observe_priority().await }
            }
        })
        .jobs(WorkerJob::RefreshPerpetualTrackedAddresses, Chain::perpetual_chains(), |chain, _| {
            let refresher = refresher.clone();
            move |_| {
                let refresher = refresher.clone();
                async move { refresher.update(chain).await }
            }
        })
        .finish()
}
