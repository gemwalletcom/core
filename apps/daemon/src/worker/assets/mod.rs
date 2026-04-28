mod asset_rank_updater;
mod assets_has_price_updater;
mod assets_images_updater;
mod perpetual_updater;
mod staking_apy_updater;
mod usage_rank_updater;
mod validator_scanner;

use std::error::Error;
use std::sync::Arc;

use api_connector::StaticAssetsClient;
use asset_rank_updater::AssetRankUpdater;
use assets_has_price_updater::AssetsHasPriceUpdater;
use assets_images_updater::AssetsImagesUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use perpetual_updater::PerpetualUpdater;
use primitives::Chain;
use settings::service_user_agent;
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use storage::ConfigCacher;
use usage_rank_updater::UsageRankUpdater;
use validator_scanner::ValidatorScanner;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    ctx.plan_builder(WorkerService::Assets, &config, shutdown_rx)
        .job(WorkerJob::UpdateSuspiciousAssetRanks, {
            let database = database.clone();
            move |_| {
                let suspicious_updater = AssetRankUpdater::new(database.clone());
                async move { suspicious_updater.update_suspicious_assets().await }
            }
        })
        .jobs(WorkerJob::UpdatePerpetuals, PerpetualUpdater::chains(), |chain, _| {
            let chain = *chain;
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                async move {
                    let updater = PerpetualUpdater::new((*settings.as_ref()).clone(), database.clone());
                    updater.update_chain(chain).await
                }
            }
        })
        .job(WorkerJob::UpdateUsageRanks, {
            let database = database.clone();
            move |_| {
                let updater = UsageRankUpdater::new(database.clone());
                async move { updater.update_usage_ranks().await }
            }
        })
        .jobs(WorkerJob::UpdateAssetsImages, Chain::all(), |chain, _| {
            let static_assets_client = StaticAssetsClient::new(&settings.assets.url);
            let database = database.clone();
            move |_| {
                let updater = AssetsImagesUpdater::new(static_assets_client.clone(), database.clone());
                async move { updater.update_chain(chain).await }
            }
        })
        .job(WorkerJob::UpdateAssetsHasPrice, {
            let database = database.clone();
            move |_| {
                let updater = AssetsHasPriceUpdater::new(database.clone());
                async move { updater.update().await }
            }
        })
        .jobs(WorkerJob::UpdateStakeApy, Chain::stakeable(), {
            let settings = settings.clone();
            let database = database.clone();
            move |chain, _| {
                let providers = Arc::new(ChainProviders::for_chain(chain, &settings, &service_user_agent("daemon", Some("staking_apy"))));
                let database = database.clone();
                move |_| {
                    let updater = StakeApyUpdater::new(providers.clone(), database.clone());
                    async move { updater.update_chain(chain).await }
                }
            }
        })
        .jobs(WorkerJob::UpdateChainValidators, Chain::stakeable(), {
            let settings = settings.clone();
            let database = database.clone();
            move |chain, _| {
                let providers = Arc::new(ChainProviders::for_chain(chain, &settings, &service_user_agent("daemon", Some("scan_validators"))));
                let database = database.clone();
                move |_| {
                    let scanner = ValidatorScanner::new(providers.clone(), database.clone());
                    async move { scanner.update_validators_for_chain(chain).await }
                }
            }
        })
        .jobs(WorkerJob::UpdateValidatorsFromStaticAssets, [Chain::Tron, Chain::SmartChain], {
            let settings = settings.clone();
            let database = database.clone();
            move |chain, _| {
                let providers = Arc::new(ChainProviders::for_chain(chain, &settings, &service_user_agent("daemon", Some("scan_static_assets"))));
                let assets_url = settings.assets.url.clone();
                let database = database.clone();
                move |_| {
                    let scanner = ValidatorScanner::new(providers.clone(), database.clone());
                    let assets_url = assets_url.clone();
                    async move { scanner.update_validators_from_static_assets_for_chain(chain, &assets_url).await }
                }
            }
        })
        .finish()
}
