pub mod alerter;
pub mod assets;
pub mod context;
pub mod device;
pub mod fiat;
pub mod job_schedule;
pub mod jobs;
pub mod plan;
pub mod pricer;
pub mod prices_dex;
pub mod rewards;
pub mod runtime;
pub mod scan;
pub mod search;
pub mod transaction;
pub mod version;

use crate::model::WorkerService;
use crate::shutdown::ShutdownReceiver;
use crate::worker::context::WorkerContext;
use job_runner::JobHandle;
use std::error::Error;

impl WorkerService {
    pub async fn run_jobs(self, ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
        match self {
            WorkerService::Alerter => alerter::jobs(ctx, shutdown_rx).await,
            WorkerService::Pricer => pricer::jobs(ctx, shutdown_rx).await,
            WorkerService::PricesDex => prices_dex::jobs(ctx, shutdown_rx).await,
            WorkerService::Fiat => fiat::jobs(ctx, shutdown_rx).await,
            WorkerService::Assets => assets::jobs(ctx, shutdown_rx).await,
            WorkerService::Version => version::jobs(ctx, shutdown_rx).await,
            WorkerService::Transaction => transaction::jobs(ctx, shutdown_rx).await,
            WorkerService::Device => device::jobs(ctx, shutdown_rx).await,
            WorkerService::Search => search::jobs(ctx, shutdown_rx).await,
            WorkerService::Scan => scan::jobs(ctx, shutdown_rx).await,
            WorkerService::Rewards => rewards::jobs(ctx, shutdown_rx).await,
        }
    }
}
