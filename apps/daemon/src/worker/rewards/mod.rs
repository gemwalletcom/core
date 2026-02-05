mod rewards_abuse_checker;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use rewards_abuse_checker::RewardsAbuseChecker;
use std::error::Error;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "check_rewards_abuse").await?;

    JobPlanBuilder::with_config(WorkerService::Rewards, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::CheckRewardsAbuse, {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move || {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let checker = RewardsAbuseChecker::new(database, stream_producer);
                    checker.check().await
                }
            }
        })
        .finish()
}
