use crate::model::WorkerService;
use crate::worker::jobs::{JobLabel, JobVariant, WorkerJob};
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
        J: Into<JobVariant>,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        let plan = self.plan.and_then(|plan| {
            let variant: JobVariant = job.into();
            if variant.worker() != self.worker {
                return Err(format!("job {} belongs to {:?} worker but builder is {:?}", variant.name(), variant.worker(), self.worker).into());
            }
            let interval = variant.resolve_interval(config)?;
            Ok(plan.job(variant.name(), interval, job_fn))
        });
        Self {
            worker: self.worker,
            plan,
            config,
        }
    }

    pub fn jobs<Items, Item, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        Builder: Fn(Item, JobVariant) -> F,
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
                let variant = JobVariant::labeled(job, item.job_label());
                let interval = variant.resolve_interval(config)?;
                let job_fn = build_job(item.clone(), variant.clone());
                Ok(current.job(variant.name(), interval, job_fn))
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
