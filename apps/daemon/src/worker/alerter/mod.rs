mod price_alerts_sender;
mod staking_rewards_notifier;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use job_runner::{JobHandle, ShutdownReceiver};
use price_alerts_sender::PriceAlertSender;
use pricer::PriceAlertClient;
use primitives::{Chain, ConfigKey};
use settings::service_user_agent;
use settings_chain::ChainProviders;
use staking_rewards_notifier::{StakeRewardsConfig, StakingRewardsNotifier};
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let cacher = CacherClient::new(&settings.redis.url).await;
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "send_price_alerts").await?;
    let stake_rewards_config = StakeRewardsConfig {
        threshold: config.get_f64(ConfigKey::AlerterStakeRewardsThreshold)?,
        lookback: config.get_duration(ConfigKey::AlerterStakeRewardsLookback)?,
    };
    let chain_providers = Arc::new(ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("stake_rewards"))));

    JobPlanBuilder::with_config(WorkerService::Alerter, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::AlertPriceAlerts, {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move |_| {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let price_alert_client = PriceAlertClient::new(database.clone());
                    PriceAlertSender::new(database, price_alert_client, stream_producer).run_observer().await
                }
            }
        })
        .jobs(WorkerJob::AlertStakeRewards, Chain::stakeable(), |chain, _| {
            let chain_providers = chain_providers.clone();
            let database = database.clone();
            let cacher = cacher.clone();
            let stream_producer = stream_producer.clone();
            move |_| {
                let chain_providers = chain_providers.clone();
                let database = database.clone();
                let cacher = cacher.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let notifier = StakingRewardsNotifier::new(chain_providers, database, stake_rewards_config, cacher, stream_producer);
                    notifier.check_chain(chain).await
                }
            }
        })
        .finish()
}
