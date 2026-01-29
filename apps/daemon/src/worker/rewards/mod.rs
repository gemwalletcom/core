mod rewards_abuse_checker;

use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use primitives::ConfigKey;
use rewards_abuse_checker::RewardsAbuseChecker;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "check_rewards_abuse").await.unwrap();

    let abuse_checker = tokio::spawn(run_job(
        "check_rewards_abuse",
        config.get_duration(ConfigKey::RewardsTimerAbuseChecker)?,
        reporter.clone(),
        shutdown_rx,
        {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move || {
                let checker = RewardsAbuseChecker::new(database.clone(), stream_producer.clone());
                async move { checker.check().await }
            }
        },
    ));

    Ok(vec![abuse_checker])
}
