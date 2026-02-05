use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use gem_tracing::{error_with_fields, info_with_fields};
pub mod schedule;
pub use schedule::{JobSchedule, RunAlways, RunDecision};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant};

pub type ShutdownReceiver = watch::Receiver<bool>;
pub type JobError = Box<dyn std::error::Error + Send + Sync>;

pub trait JobStatusReporter: Send + Sync {
    fn report(&self, name: &str, interval: u64, duration: u64, success: bool, error: Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

pub async fn sleep_or_shutdown(duration: Duration, shutdown_rx: &ShutdownReceiver) -> bool {
    let mut rx = shutdown_rx.clone();
    tokio::select! {
        _ = tokio::time::sleep(duration) => false,
        _ = rx.changed() => true,
    }
}

fn human_duration(duration: Duration) -> String {
    if duration.is_zero() {
        return "0s".to_string();
    }

    let mut parts = Vec::new();
    let mut remaining = duration.as_secs();
    const UNITS: [(&str, u64); 4] = [("d", 86_400), ("h", 3_600), ("m", 60), ("s", 1)];

    for (label, unit) in UNITS {
        if remaining >= unit {
            let value = remaining / unit;
            remaining %= unit;
            parts.push(format!("{value}{label}"));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if parts.is_empty() { format!("{}ms", duration.subsec_millis()) } else { parts.join(" ") }
}

pub async fn run_job<Name, F, Fut, R>(
    name: Name,
    interval_duration: Duration,
    reporter: Arc<dyn JobStatusReporter>,
    shutdown_rx: ShutdownReceiver,
    schedule: Arc<dyn JobSchedule>,
    job_fn: F,
) where
    Name: Into<String> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, JobError>> + Send + 'static,
    R: Debug + Send + Sync + 'static,
{
    let job_name = name.into();

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        let decision = schedule.evaluate(job_name.as_str(), interval_duration, SystemTime::now()).await;
        match decision {
            Ok(RunDecision::Run) => {}
            Ok(RunDecision::Wait(wait)) if wait > Duration::ZERO => {
                info_with_fields!("job wait", job = job_name.as_str(), wait = human_duration(wait));
                if sleep_or_shutdown(wait, &shutdown_rx).await {
                    break;
                }
                continue;
            }
            Ok(RunDecision::Wait(_)) => {}
            Err(err) => {
                error_with_fields!("job schedule evaluation failed", &*err, job = job_name.as_str());
            }
        }

        let now = Instant::now();
        info_with_fields!("job start", job = job_name.as_str(), interval = human_duration(interval_duration));

        let result = job_fn().await;
        let duration_ms = now.elapsed().as_millis() as u64;
        let duration_display = human_duration(Duration::from_millis(duration_ms));

        match result {
            Ok(value) => {
                info_with_fields!(
                    "job complete",
                    job = job_name.as_str(),
                    duration = duration_display.as_str(),
                    result = format!("{:?}", value)
                );
                if let Err(err) = schedule.mark_success(job_name.as_str(), SystemTime::now()).await {
                    error_with_fields!("job schedule update failed", &*err, job = job_name.as_str());
                }
                reporter.report(&job_name, interval_duration.as_secs(), duration_ms, true, None).await;
            }
            Err(err) => {
                let mut msg = format!("{:?}", err);
                if msg.len() > 200 {
                    msg.truncate(200);
                }
                error_with_fields!("job failed", &*err, job = job_name.as_str(), duration = duration_display.as_str());
                reporter.report(&job_name, interval_duration.as_secs(), duration_ms, false, Some(msg)).await;
            }
        }

        if *shutdown_rx.borrow() || sleep_or_shutdown(interval_duration, &shutdown_rx).await {
            break;
        }
    }
}

pub struct JobPlan {
    reporter: Arc<dyn JobStatusReporter>,
    shutdown_rx: ShutdownReceiver,
    schedule: Arc<dyn JobSchedule>,
    handles: Vec<JobHandle>,
}

impl JobPlan {
    pub fn new(reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Self {
        Self::with_history(reporter, shutdown_rx, Arc::new(RunAlways))
    }

    pub fn with_history(reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver, schedule: Arc<dyn JobSchedule>) -> Self {
        Self {
            reporter,
            shutdown_rx,
            schedule,
            handles: Vec::new(),
        }
    }

    pub fn job<Name, F, Fut, R>(mut self, name: Name, interval: Duration, job_fn: F) -> Self
    where
        Name: Into<String> + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let job_name = name.into();
        let finished = Arc::new(AtomicBool::new(false));
        let finished_clone = finished.clone();
        let reporter = self.reporter.clone();
        let shutdown_rx = self.shutdown_rx.clone();
        let schedule = self.schedule.clone();
        let job_name_for_handle = job_name.clone();
        let handle = tokio::spawn(async move {
            run_job(job_name_for_handle, interval, reporter, shutdown_rx, schedule, job_fn).await;
            finished_clone.store(true, Ordering::Relaxed);
        });
        self.handles.push(JobHandle::new(job_name, handle, finished));
        self
    }

    pub fn finish(self) -> Vec<JobHandle> {
        self.handles
    }
}

pub struct JobHandle {
    name: String,
    handle: JoinHandle<()>,
    finished: Arc<AtomicBool>,
}

impl JobHandle {
    pub fn new(name: String, handle: JoinHandle<()>, finished: Arc<AtomicBool>) -> Self {
        Self { name, handle, finished }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn status_flag(&self) -> Arc<AtomicBool> {
        self.finished.clone()
    }

    pub fn into_handle(self) -> JoinHandle<()> {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    use super::human_duration;
    use std::time::Duration;

    #[test]
    fn duration_zero() {
        assert_eq!(human_duration(Duration::ZERO), "0s");
    }

    #[test]
    fn duration_sub_second() {
        assert_eq!(human_duration(Duration::from_millis(250)), "250ms");
    }

    #[test]
    fn duration_seconds_and_minutes() {
        assert_eq!(human_duration(Duration::from_secs(12)), "12s");
        assert_eq!(human_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(human_duration(Duration::from_secs(65)), "1m 5s");
    }

    #[test]
    fn duration_hours_and_days() {
        assert_eq!(human_duration(Duration::from_secs(3_600 * 5 + 42)), "5h 42s");
        assert_eq!(human_duration(Duration::from_secs(86_400 + 3_600 * 2)), "1d 2h");
        assert_eq!(human_duration(Duration::from_secs(90_000)), "1d 1h");
    }
}
