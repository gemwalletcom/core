mod rewards_abuse_checker;
mod rewards_eligibility_checker;

use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};
use rewards_abuse_checker::RewardsAbuseChecker;
use rewards_eligibility_checker::RewardsEligibilityChecker;
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
    let stream_producer = StreamProducer::new(&rabbitmq_config, "rewards_worker", shutdown_rx.clone()).await?;

    ctx.plan_builder(WorkerService::Rewards, &config, shutdown_rx)
        .job(WorkerJob::CheckRewardsAbuse, {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move |_| {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let checker = RewardsAbuseChecker::new(database, stream_producer);
                    checker.check().await
                }
            }
        })
        .job(WorkerJob::CheckRewardsEligibility, {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move |_| {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let checker = RewardsEligibilityChecker::new(database, stream_producer);
                    checker.check().await
                }
            }
        })
        .finish()
}
