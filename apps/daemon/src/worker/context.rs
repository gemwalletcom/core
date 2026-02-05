use crate::worker::runtime::WorkerRuntime;
use settings::Settings;
use std::sync::Arc;
use storage::Database;

#[derive(Clone)]
pub struct WorkerContext {
    settings: Arc<Settings>,
    database: Database,
    runtime: WorkerRuntime,
}

impl WorkerContext {
    pub fn new(settings: Arc<Settings>, database: Database, runtime: WorkerRuntime) -> Self {
        Self { settings, database, runtime }
    }

    pub fn settings(&self) -> Arc<Settings> {
        self.settings.clone()
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn runtime(&self) -> WorkerRuntime {
        self.runtime.clone()
    }
}
