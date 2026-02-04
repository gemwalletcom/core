use crate::model::WorkerService;
use crate::worker::jobs::{JobInstance, WorkerJob};
use job_runner::{JobError, JobHandle, JobPlan};
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use storage::ConfigCacher;

type PlanResult = Result<JobPlan, Box<dyn Error + Send + Sync>>;

pub struct JobPlanBuilder<'a> {
    worker: WorkerService,
    plan: PlanResult,
    config: Option<&'a ConfigCacher>,
}

impl<'a> JobPlanBuilder<'a> {
    pub fn new(worker: WorkerService, plan: JobPlan) -> Self {
        Self {
            worker,
            plan: Ok(plan),
            config: None,
        }
    }

    pub fn with_config(worker: WorkerService, plan: JobPlan, config: &'a ConfigCacher) -> Self {
        Self {
            worker,
            plan: Ok(plan),
            config: Some(config),
        }
    }

    pub fn job<J, F, Fut, R>(self, job: J, job_fn: F) -> Self
    where
        J: Into<JobInstance>,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        let plan = self.plan.and_then(|plan| {
            let instance: JobInstance = job.into();
            if instance.worker() != self.worker {
                return Err(format!("job {} belongs to {:?} worker but builder is {:?}", instance.name(), instance.worker(), self.worker).into());
            }
            let interval = instance.resolve_interval(config)?;
            Ok(plan.job(instance.name().to_string(), interval, job_fn))
        });
        Self {
            worker: self.worker,
            plan,
            config,
        }
    }

    pub fn jobs<T, Item, Labeler, Build, F, Fut, R>(self, job: WorkerJob, items: T, labeler: Labeler, build_job: Build) -> Self
    where
        T: IntoIterator<Item = Item>,
        Item: Clone + Send + Sync + 'static,
        Labeler: Fn(&Item) -> String,
        Build: Fn(Item, JobInstance) -> F,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        let plan = self.plan.and_then(|plan| {
            if job.worker() != self.worker {
                return Err(format!("job {} belongs to {:?} worker but builder is {:?}", job.as_ref(), job.worker(), self.worker).into());
            }
            items.into_iter().try_fold(plan, |current, item| {
                let label = labeler(&item);
                let instance = JobInstance::labeled(job, label);
                let interval = instance.resolve_interval(config)?;
                let job_fn = build_job(item.clone(), instance.clone());
                Ok(current.job(instance.name().to_string(), interval, job_fn))
            })
        });
        Self {
            worker: self.worker,
            plan,
            config,
        }
    }

    pub fn finish(self) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
        self.plan.map(JobPlan::finish)
    }
}
