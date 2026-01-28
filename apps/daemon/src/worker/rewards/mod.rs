mod rewards_abuse_checker;

use job_runner::{ShutdownReceiver, run_job};
use primitives::ConfigKey;
use rewards_abuse_checker::RewardsAbuseChecker;
use settings::Settings;
use std::error::Error;
use storage::ConfigCacher;
use streamer::StreamProducer;
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "rewards_abuse_checker").await.unwrap();

    let abuse_checker = tokio::spawn(run_job("rewards abuse checker", config.get_duration(ConfigKey::RewardsTimerAbuseChecker)?, shutdown_rx, {
        let database = database.clone();
        let stream_producer = stream_producer.clone();
        move || {
            let checker = RewardsAbuseChecker::new(database.clone(), stream_producer.clone());
            async move { checker.check().await }
        }
    }));

    Ok(vec![abuse_checker])
}
