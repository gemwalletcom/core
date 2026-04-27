use crate::model::WorkerService;
use crate::shutdown::ShutdownReceiver;
use crate::worker::plan::JobPlanBuilder;
use crate::worker::runtime::WorkerRuntime;
use settings::Settings;
use std::sync::Arc;
use storage::{ConfigCacher, Database};

#[derive(Clone)]
pub struct WorkerContext {
    settings: Arc<Settings>,
    database: Database,
    runtime: WorkerRuntime,
    job_filter: Option<String>,
}

impl WorkerContext {
    pub fn new(settings: Arc<Settings>, database: Database, runtime: WorkerRuntime, job_filter: Option<String>) -> Self {
        Self {
            settings,
            database,
            runtime,
            job_filter,
        }
    }

    pub fn settings(&self) -> Arc<Settings> {
        self.settings.clone()
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn plan_builder<'a>(&self, worker: WorkerService, config: &'a ConfigCacher, shutdown_rx: ShutdownReceiver) -> JobPlanBuilder<'a> {
        JobPlanBuilder::with_config(worker, self.runtime.plan(shutdown_rx), config).filter(self.job_filter.clone())
    }
}
