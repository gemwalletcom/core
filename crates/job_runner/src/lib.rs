use chrono::Local;
use std::future::Future;
use tokio::time::{Duration, Instant};

pub fn run_job<F, Fut, R>(name: &'static str, interval_duration: Duration, job_fn: F) -> impl Future<Output = ()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: std::fmt::Debug + Send + 'static,
{
    async move {
        loop {
            let now = Instant::now();

            println!(
                "{}: start: {}, interval: {}s",
                Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                name,
                interval_duration.as_secs()
            );

            let result = job_fn().await;

            println!(
                "{}: done in {}ms: {}. Result: {:?}",
                Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                now.elapsed().as_millis(),
                name,
                result
            );

            tokio::time::sleep(interval_duration).await;
        }
    }
}
