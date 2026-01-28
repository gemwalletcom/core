use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use gem_tracing::info_with_fields;
use primitives::JobStatus;
use tokio::sync::watch;
use tokio::time::{Duration, Instant};

pub type ShutdownReceiver = watch::Receiver<bool>;

pub trait JobResult {
    fn is_success(&self) -> bool;
    fn error_message(&self) -> Option<String>;
}

impl<T: Debug, E: Debug> JobResult for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }

    fn error_message(&self) -> Option<String> {
        self.as_ref().err().map(|e| format!("{:?}", e))
    }
}

pub trait JobStatusReporter: Send + Sync {
    fn report(&self, name: &str, status: JobStatus) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    fn get_status(&self, name: &str) -> Pin<Box<dyn Future<Output = Option<JobStatus>> + Send + '_>>;
}

pub async fn sleep_or_shutdown(duration: Duration, shutdown_rx: &ShutdownReceiver) -> bool {
    let mut rx = shutdown_rx.clone();
    tokio::select! {
        _ = tokio::time::sleep(duration) => false,
        _ = rx.changed() => true,
    }
}

pub async fn run_job<F, Fut, R>(name: &'static str, interval_duration: Duration, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver, job_fn: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: JobResult + Debug + Send + Sync + 'static,
{
    loop {
        if *shutdown_rx.borrow() {
            info_with_fields!("job shutdown", job = name);
            break;
        }

        let now = Instant::now();

        info_with_fields!("job start", job = name, interval = interval_duration.as_secs().to_string());

        let result = job_fn().await;
        let duration_ms = now.elapsed().as_millis() as u64;

        info_with_fields!("job complete", job = name, duration = format!("{}ms", duration_ms), result = format!("{:?}", result));

        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        let mut status = reporter.get_status(name).await.unwrap_or_default();
        status.interval = interval_duration.as_secs();
        status.duration = duration_ms;

        if result.is_success() {
            status.last_success = Some(timestamp);
        } else if let Some(msg) = result.error_message() {
            let truncated = if msg.len() > 200 { msg[..200].to_string() } else { msg };
            status.last_error = Some(truncated);
            status.last_error_at = Some(timestamp);
        }

        reporter.report(name, status).await;

        if *shutdown_rx.borrow() {
            info_with_fields!("job shutdown", job = name);
            break;
        }

        if sleep_or_shutdown(interval_duration, &shutdown_rx).await {
            info_with_fields!("job shutdown", job = name);
            break;
        }
    }
}
