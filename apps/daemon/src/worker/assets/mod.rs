mod asset_rank_updater;
pub mod asset_updater;
mod assets_images_updater;
mod perpetual_updater;
mod staking_apy_updater;
mod usage_rank_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use api_connector::StaticAssetsClient;
use asset_rank_updater::AssetRankUpdater;
use asset_updater::AssetUpdater;
use assets_images_updater::AssetsImagesUpdater;
use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use job_runner::{JobHandle, ShutdownReceiver};
use perpetual_updater::PerpetualUpdater;
use primitives::Chain;
use settings::service_user_agent;
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use std::error::Error;
use storage::ConfigCacher;
use usage_rank_updater::UsageRankUpdater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = ConfigCacher::new(database.clone());

    let asset_updater = {
        let coingecko_client = coingecko_client.clone();
        let database = database.clone();
        let cacher_client = cacher_client.clone();
        move || AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone())
    };

    JobPlanBuilder::with_config(WorkerService::Assets, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::UpdateExistingPricesAssets, {
            let asset_updater = asset_updater.clone();
            move || {
                let updater = asset_updater();
                async move { updater.update_existing_assets().await }
            }
        })
        .job(WorkerJob::UpdateAllPricesAssets, {
            let asset_updater = asset_updater.clone();
            move || {
                let updater = asset_updater();
                async move { updater.update_assets().await }
            }
        })
        .job(WorkerJob::UpdateNativePricesAssets, {
            let asset_updater = asset_updater.clone();
            move || {
                let updater = asset_updater();
                async move { updater.update_native_prices_assets().await }
            }
        })
        .job(WorkerJob::UpdateCoingeckoTrendingAssets, {
            let asset_updater = asset_updater.clone();
            move || {
                let updater = asset_updater();
                async move { updater.update_trending_assets().await }
            }
        })
        .job(WorkerJob::UpdateCoingeckoRecentlyAddedAssets, {
            let asset_updater = asset_updater.clone();
            move || {
                let updater = asset_updater();
                async move { updater.update_recently_added_assets().await }
            }
        })
        .job(WorkerJob::UpdateSuspiciousAssetRanks, {
            let database = database.clone();
            move || {
                let suspicious_updater = AssetRankUpdater::new(database.clone());
                async move { suspicious_updater.update_suspicious_assets().await }
            }
        })
        .jobs(
            WorkerJob::UpdatePerpetuals,
            PerpetualUpdater::chains(),
            |chain| chain.as_ref().to_string(),
            |chain, _| {
                let chain = *chain;
                let settings = settings.clone();
                let database = database.clone();
                move || {
                    let settings = settings.clone();
                    let database = database.clone();
                    async move {
                        let updater = PerpetualUpdater::new((*settings.as_ref()).clone(), database.clone());
                        updater.update_chain(chain).await
                    }
                }
            },
        )
        .job(WorkerJob::UpdateUsageRanks, {
            let database = database.clone();
            move || {
                let updater = UsageRankUpdater::new(database.clone());
                async move { updater.update_usage_ranks().await }
            }
        })
        .job(WorkerJob::UpdateAssetsImages, {
            let static_assets_client = StaticAssetsClient::new(&settings.assets.url);
            let database = database.clone();
            move || {
                let updater = AssetsImagesUpdater::new(static_assets_client.clone(), database.clone());
                async move { updater.update_assets_images().await }
            }
        })
        .jobs(
            WorkerJob::UpdateStakingApy,
            Chain::stakeable(),
            |chain| chain.as_ref().to_string(),
            |chain, _| {
                let chain = chain;
                let settings = settings.clone();
                let database = database.clone();
                move || {
                    let chain = chain;
                    let settings = settings.clone();
                    let database = database.clone();
                    async move {
                        let providers = ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("staking_apy")));
                        let updater = StakeApyUpdater::new(providers, database.clone());
                        updater.update_chain(chain).await
                    }
                }
            },
        )
        .finish()
}
