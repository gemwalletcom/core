use tokio::task;
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant};

pub fn run_job<F, Fut>(name: &'static str, interval_duration: Duration, job_fn: F) -> JoinHandle<()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    task::spawn(async move {
        loop {
            let now = Instant::now();

            println!("Job start: {}, interval: {} seconds", name, interval_duration.as_secs());

            job_fn().await;

            println!("Job done in {} seconds: {}", now.elapsed().as_secs(), name);

            tokio::time::sleep(interval_duration).await;
        }
    })
}
