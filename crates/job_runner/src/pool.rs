use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::spawn;
use tokio::task::AbortHandle;
use tokio::time::{Instant, sleep_until};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Complete,
    Retry,
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveConfig {
    pub initial: Duration,
    pub max: Duration,
    pub step: f64,
}

impl AdaptiveConfig {
    pub fn new(initial: Duration, max: Duration, step: f64) -> Self {
        Self { initial: initial.max(Duration::from_millis(1)).min(max), max, step }
    }

    pub fn next_interval(&self, current: Duration) -> Duration {
        current.mul_f64(self.step).clamp(self.initial, self.max)
    }
}

#[derive(Debug, Clone)]
pub enum JobConfig {
    Fixed { interval: Duration, deadline: Option<Duration> },
    Adaptive { config: AdaptiveConfig, deadline: Option<Duration> },
}

impl JobConfig {
    pub fn initial_interval(&self) -> Duration {
        match self {
            Self::Fixed { interval, .. } => *interval,
            Self::Adaptive { config, .. } => config.initial,
        }
    }

    pub fn deadline(&self) -> Option<Duration> {
        match self {
            Self::Fixed { deadline, .. } | Self::Adaptive { deadline, .. } => *deadline,
        }
    }

    pub fn next_interval(&self, current: Duration) -> Duration {
        match self {
            Self::Fixed { interval, .. } => *interval,
            Self::Adaptive { config, .. } => config.next_interval(current),
        }
    }
}

#[async_trait]
pub trait Job: Send + Sync + 'static {
    fn id(&self) -> String;
    fn config(&self) -> JobConfig;
    async fn run(&self) -> JobStatus;
}

struct Task {
    spawn_id: u64,
    abort_handle: AbortHandle,
}

impl Task {
    fn abort(&self) {
        self.abort_handle.abort();
    }
}

pub struct JobRunner {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    spawn_counter: AtomicU64,
}

impl Default for JobRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl JobRunner {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            spawn_counter: AtomicU64::new(0),
        }
    }

    pub async fn spawn(&self, job: Arc<dyn Job>) {
        let id = job.id();
        let spawn_id = self.spawn_counter.fetch_add(1, Ordering::Relaxed);

        self.remove_task(&id).inspect(Task::abort);

        let tasks_for_cleanup = self.tasks.clone();
        let cleanup_id = id.clone();
        let handle = spawn(async move {
            run_loop(job).await;
            if let Ok(mut tasks) = tasks_for_cleanup.lock()
                && tasks.get(&cleanup_id).is_some_and(|entry| entry.spawn_id == spawn_id)
            {
                tasks.remove(&cleanup_id);
            }
        });

        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.insert(id, Task { spawn_id, abort_handle: handle.abort_handle() });
        }
    }

    pub async fn cancel(&self, id: &str) {
        self.remove_task(id).inspect(Task::abort);
    }

    pub async fn stop_all(&self) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.drain().for_each(|(_, entry)| entry.abort());
        }
    }

    fn remove_task(&self, id: &str) -> Option<Task> {
        self.tasks.lock().ok().and_then(|mut tasks| tasks.remove(id))
    }
}

impl Drop for JobRunner {
    fn drop(&mut self) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.drain().for_each(|(_, entry)| entry.abort());
        }
    }
}

async fn run_loop(job: Arc<dyn Job>) {
    let config = job.config();
    let job_start = Instant::now();
    let mut interval = config.initial_interval();

    loop {
        if config.deadline().is_some_and(|x: Duration| job_start.elapsed() >= x) {
            return;
        }

        let attempt_start = Instant::now();
        match job.run().await {
            JobStatus::Complete => return,
            JobStatus::Retry => {
                let retry_at = attempt_start + interval;
                let retry_at = match config.deadline() {
                    Some(deadline) => retry_at.min(job_start + deadline),
                    None => retry_at,
                };
                sleep_until(retry_at).await;
                interval = config.next_interval(interval);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_next_interval() {
        let config = AdaptiveConfig::new(Duration::from_millis(100), Duration::from_millis(500), 2.0);
        assert_eq!(config.next_interval(Duration::from_millis(100)).as_millis(), 200);
        assert_eq!(config.next_interval(Duration::from_millis(200)).as_millis(), 400);
        assert_eq!(config.next_interval(Duration::from_millis(400)).as_millis(), 500);
    }
}
