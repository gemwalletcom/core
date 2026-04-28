use crate::model::WorkerService;
use crate::worker::jobs::{JobLabel, JobVariant, WorkerJob};
use job_runner::{JobContext, JobError, JobHandle, JobPlan};
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use storage::ConfigCacher;

type PlanResult = Result<JobPlan, Box<dyn Error + Send + Sync>>;

pub struct JobPlanBuilder<'a> {
    worker: WorkerService,
    plan: PlanResult,
    config: Option<&'a ConfigCacher>,
    filter: Option<String>,
}

impl<'a> JobPlanBuilder<'a> {
    pub fn with_config(worker: WorkerService, plan: JobPlan, config: &'a ConfigCacher) -> Self {
        Self {
            worker,
            plan: Ok(plan),
            config: Some(config),
            filter: None,
        }
    }

    pub fn filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    pub fn job<J, F, Fut, R>(self, job: J, job_fn: F) -> Self
    where
        J: Into<JobVariant>,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        let filter = self.filter.clone();
        let plan = self.plan.and_then(|plan| {
            let variant: JobVariant = job.into();
            if variant.worker() != self.worker {
                return Err(format!("job {} belongs to {:?} worker but builder is {:?}", variant.name(), variant.worker(), self.worker).into());
            }
            if !should_include(&variant.name(), filter.as_deref()) {
                return Ok(plan);
            }
            let interval = variant.resolve_interval(config)?;
            Ok(plan.job(variant.name(), interval, job_fn))
        });
        Self {
            worker: self.worker,
            plan,
            config,
            filter: self.filter,
        }
    }

    pub fn jobs<Items, Item, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        self.build_jobs(job, items, |_, _| Ok(None), build_job)
    }

    pub fn jobs_with_config<Items, Item, K, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, config_key: K, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        K: Fn(Item) -> primitives::ConfigParamKey,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        self.build_jobs(
            job,
            items,
            move |item, _| {
                let param = config_key(item);
                let cfg = config.ok_or("ConfigCacher required for jobs_with_config")?;
                Ok(Some(cfg.get_param_duration(&param)?))
            },
            build_job,
        )
    }

    fn build_jobs<Items, Item, V, Builder, F, Fut, R>(self, job: WorkerJob, items: Items, modify_interval: V, build_job: Builder) -> Self
    where
        Items: IntoIterator<Item = Item>,
        Item: JobLabel + Clone + Send + Sync + 'static,
        V: Fn(Item, &JobVariant) -> Result<Option<Duration>, Box<dyn Error + Send + Sync>>,
        Builder: Fn(Item, JobVariant) -> F,
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, JobError>> + Send + 'static,
        R: Debug + Send + Sync + 'static,
    {
        let config = self.config;
        let filter = self.filter.clone();
        let plan = self.plan.and_then(|plan| {
            if job.worker() != self.worker {
                return Err(format!("job {} belongs to {:?} worker but builder is {:?}", job.as_ref(), job.worker(), self.worker).into());
            }
            items.into_iter().try_fold(plan, |current, item| {
                let variant = JobVariant::labeled(job, item.job_label());
                let variant = match modify_interval(item.clone(), &variant)? {
                    Some(duration) => variant.every(duration),
                    None => variant,
                };
                if !should_include(&variant.name(), filter.as_deref()) {
                    return Ok(current);
                }
                let interval = variant.resolve_interval(config)?;
                let job_fn = build_job(item, variant.clone());
                Ok(current.job(variant.name(), interval, job_fn))
            })
        });
        Self {
            worker: self.worker,
            plan,
            config,
            filter: self.filter,
        }
    }

    pub fn finish(self) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
        self.plan.map(JobPlan::finish)
    }
}

fn should_include(name: &str, filter: Option<&str>) -> bool {
    filter.is_none_or(|f| f.is_empty() || name.contains(f))
}

#[cfg(test)]
mod tests {
    use super::should_include;

    #[test]
    fn test_should_include() {
        assert!(should_include("publish_missing_prices", None));
        assert!(should_include("publish_missing_prices", Some("")));
        assert!(should_include("publish_missing_prices", Some("missing")));
        assert!(should_include("update_prices_top.coingecko", Some("coingecko")));
        assert!(!should_include("update_prices_top.coingecko", Some("pyth")));
        assert!(!should_include("publish_missing_prices", Some("update_prices")));
    }
}
