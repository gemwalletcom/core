use std::future::Future;
use std::pin::Pin;

use cacher::CacherClient;
use gem_tracing::info_with_fields;
use job_runner::JobStatusReporter;
use primitives::JobStatus;

const JOBS_STATUS_PREFIX: &str = "jobs:status:";

pub struct CacherJobReporter {
    cacher: CacherClient,
}

impl CacherJobReporter {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }
}

fn normalize_key(name: &str) -> String {
    format!("{}{}", JOBS_STATUS_PREFIX, name.to_lowercase().replace(' ', "_"))
}

impl JobStatusReporter for CacherJobReporter {
    fn report(&self, name: &str, status: JobStatus) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let key = normalize_key(name);
        Box::pin(async move {
            if let Err(e) = self.cacher.set_value(&key, &status).await {
                info_with_fields!("job status report failed", job = key, error = format!("{:?}", e));
            }
        })
    }

    fn get_status(&self, name: &str) -> Pin<Box<dyn Future<Output = Option<JobStatus>> + Send + '_>> {
        let key = normalize_key(name);
        Box::pin(async move { self.cacher.get_value(&key).await.ok() })
    }
}
