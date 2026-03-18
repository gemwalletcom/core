mod in_transit_updater;
mod perpetual_address_refresher;
mod perpetual_classifier;
mod perpetual_observer;
mod vault_addresses_updater;

use cacher::CacherClient;
use in_transit_updater::{InTransitConfig, InTransitUpdater};
use job_runner::{JobHandle, ShutdownReceiver};
use perpetual_address_refresher::PerpetualAddressRefresher;
use perpetual_classifier::{PerpetualPositionClassifier, PerpetualPriorityConfig};
use perpetual_observer::PerpetualPositionObserver;
use primitives::{Chain, ConfigKey, ParamConfigKey, SwapProvider};
use settings_chain::{ChainProviders, ProviderFactory};
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use swapper::NativeProvider;
use swapper::swapper::GemSwapper;
use vault_addresses_updater::VaultAddressesUpdater;

use crate::client::SwapVaultAddressClient;
use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;

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
    let stream_producer = StreamProducer::new(&rabbitmq_config, "transactions_worker", shutdown_rx.clone()).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;

    let perpetual_providers = Arc::new(ChainProviders::from_settings(
        &settings,
        &settings::service_user_agent("daemon", Some("perpetual_observer")),
    ));
    let priority_config = PerpetualPriorityConfig {
        trigger_bps: config.get_i64(ConfigKey::TransactionPerpetualPriorityTriggerBps)?,
        liquidation_bps: config.get_i64(ConfigKey::TransactionPerpetualPriorityLiquidationBps)?,
    };
    let refresher = Arc::new(PerpetualAddressRefresher::new(perpetual_providers.clone(), database.clone(), cacher.clone()));

    JobPlanBuilder::with_config(WorkerService::Transactions, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateInTransitTransactions, {
            let database = database.clone();
            let swapper = swapper.clone();
            let stream_producer = stream_producer.clone();
            let vault_client = SwapVaultAddressClient::new(cacher.clone());
            move |_| {
                let updater = InTransitUpdater::new(database.clone(), in_transit_config, swapper.clone(), stream_producer.clone(), vault_client.clone());
                async move { updater.update().await }
            }
        })
        .jobs(WorkerJob::ClassifyPerpetualAddresses, Chain::perpetual_chains(), |chain, _| {
            let classifier = Arc::new(PerpetualPositionClassifier::new(chain, perpetual_providers.clone(), cacher.clone(), priority_config));
            move |_| {
                let classifier = classifier.clone();
                async move { classifier.classify().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualActiveAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(PerpetualPositionObserver::new(chain, perpetual_providers.clone(), cacher.clone(), stream_producer.clone()));
            move |_| {
                let observer = observer.clone();
                async move { observer.observe_active().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualPriorityAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(PerpetualPositionObserver::new(chain, perpetual_providers.clone(), cacher.clone(), stream_producer.clone()));
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
        .jobs_with_config(
            WorkerJob::UpdateSwapVaultAddresses,
            SwapProvider::cross_chain_providers(),
            ParamConfigKey::SwapperVaultAddresses,
            |provider, _| {
                let updater = Arc::new(VaultAddressesUpdater::new(swapper.clone(), cacher.clone()));
                move |ctx| {
                    let updater = updater.clone();
                    async move { updater.update(provider, ctx.last_success_at).await }
                }
            },
        )
        .finish()
}
