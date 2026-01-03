mod rewards_abuse_checker;

use job_runner::run_job;
use rewards_abuse_checker::RewardsAbuseChecker;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use streamer::StreamProducer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "rewards_abuse_checker").await.unwrap();

    let abuse_checker = run_job("rewards abuse checker", Duration::from_secs(60), {
        let database = database.clone();
        let stream_producer = stream_producer.clone();
        move || {
            let checker = RewardsAbuseChecker::new(database.clone(), stream_producer.clone());
            async move { checker.check().await }
        }
    });

    vec![Box::pin(abuse_checker)]
}
