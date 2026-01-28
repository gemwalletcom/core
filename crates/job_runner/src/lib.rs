use gem_tracing::info_with_fields;
use std::future::Future;
use tokio::sync::watch;
use tokio::time::{Duration, Instant};

pub type ShutdownReceiver = watch::Receiver<bool>;

pub async fn run_job<F, Fut, R>(name: &'static str, interval_duration: Duration, shutdown_rx: ShutdownReceiver, job_fn: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: std::fmt::Debug + Send + Sync + 'static,
{
    loop {
        if *shutdown_rx.borrow() {
            info_with_fields!("job shutdown", job = name);
            break;
        }

        let now = Instant::now();

        info_with_fields!("job start", job = name, interval = interval_duration.as_secs().to_string());

        let result = job_fn().await;

        info_with_fields!(
            "job complete",
            job = name,
            duration = format!("{}ms", now.elapsed().as_millis()),
            result = format!("{:?}", result)
        );

        if *shutdown_rx.borrow() {
            info_with_fields!("job shutdown", job = name);
            break;
        }

        let mut rx = shutdown_rx.clone();
        tokio::select! {
            _ = tokio::time::sleep(interval_duration) => {}
            _ = rx.changed() => {
                info_with_fields!("job shutdown", job = name);
                break;
            }
        }
    }
}
