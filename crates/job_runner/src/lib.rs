use gem_tracing::info_with_fields;
use std::future::Future;
use tokio::time::{Duration, Instant};

pub async fn run_job<F, Fut, R>(name: &'static str, interval_duration: Duration, job_fn: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: std::fmt::Debug + Send + Sync + 'static,
{
    loop {
        let now = Instant::now();

        info_with_fields!("job start", job = name, interval = interval_duration.as_secs().to_string());

        let _result = job_fn().await;

        info_with_fields!("job complete", job = name, duration_ms = now.elapsed().as_millis().to_string());

        tokio::time::sleep(interval_duration).await;
    }
}
