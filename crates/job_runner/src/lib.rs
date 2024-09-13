use tokio::time::{Duration, Instant};

pub async fn run_job<F, Fut>(name: &'static str, interval_duration: Duration, job_fn: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    loop {
        let now = Instant::now();

        println!("Job start: {}, interval: {} seconds", name, interval_duration.as_secs());

        job_fn().await;

        println!("Job done in {} seconds: {}", now.elapsed().as_secs(), name);

        tokio::time::sleep(interval_duration).await;
    }
}
