mod validator_scanner;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::Chain;
use settings::service_user_agent;
use settings_chain::ChainProviders;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use validator_scanner::ValidatorScanner;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let assets_url = Arc::new(settings.assets.url.clone());

    let validator_providers = Arc::new(ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_validators"))));
    let static_providers = Arc::new(ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_static_assets"))));

    JobPlanBuilder::with_config(WorkerService::Scan, runtime.plan(shutdown_rx), &config)
        .jobs(WorkerJob::UpdateChainValidators, Chain::stakeable(), |chain, _| {
            let providers = validator_providers.clone();
            let database = database.clone();
            move || {
                let providers = providers.clone();
                let database = database.clone();
                async move {
                    let scanner = ValidatorScanner::new(providers, database);
                    scanner.update_validators_for_chain(chain).await
                }
            }
        })
        .jobs(WorkerJob::UpdateValidatorsFromStaticAssets, [Chain::Tron, Chain::SmartChain], |chain, _| {
            let providers = static_providers.clone();
            let database = database.clone();
            let assets_url = assets_url.clone();
            move || {
                let providers = providers.clone();
                let database = database.clone();
                let assets_url = assets_url.clone();
                async move {
                    let scanner = ValidatorScanner::new(providers, database);
                    scanner.update_validators_from_static_assets_for_chain(chain, assets_url.as_str()).await
                }
            }
        })
        .finish()
}
