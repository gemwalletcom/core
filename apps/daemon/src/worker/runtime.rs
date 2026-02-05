use job_runner::{JobPlan, JobSchedule, JobStatusReporter, ShutdownReceiver};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerRuntime {
    reporter: Arc<dyn JobStatusReporter>,
    schedule: Arc<dyn JobSchedule>,
}

impl WorkerRuntime {
    pub fn new(reporter: Arc<dyn JobStatusReporter>, schedule: Arc<dyn JobSchedule>) -> Self {
        Self { reporter, schedule }
    }

    pub fn plan(&self, shutdown_rx: ShutdownReceiver) -> JobPlan {
        JobPlan::with_history(self.reporter.clone(), shutdown_rx, self.schedule.clone())
    }
}
