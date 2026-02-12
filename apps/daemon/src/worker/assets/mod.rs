mod asset_rank_updater;
pub mod asset_updater;
mod assets_images_updater;
mod perpetual_updater;
mod staking_apy_updater;
mod usage_rank_updater;
mod validator_scanner;

use std::error::Error;
use std::sync::Arc;

use api_connector::StaticAssetsClient;
use asset_rank_updater::AssetRankUpdater;
use asset_updater::{AssetUpdater, AssetUpdaterConfig};
use assets_images_updater::AssetsImagesUpdater;
use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use job_runner::{JobHandle, ShutdownReceiver};
use perpetual_updater::PerpetualUpdater;
use primitives::{Chain, ConfigKey};
use settings::service_user_agent;
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use usage_rank_updater::UsageRankUpdater;
use validator_scanner::ValidatorScanner;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = ConfigCacher::new(database.clone());

    let updater_config = AssetUpdaterConfig {
        new_interval: config.get_duration(ConfigKey::AssetsUpdateNewCoinInfoInterval)?,
        existing_interval: config.get_duration(ConfigKey::AssetsUpdateExistingCoinInfoInterval)?,
    };
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "assets_worker").await?;

    let asset_updater = {
        let coingecko_client = coingecko_client.clone();
        let database = database.clone();
        let cacher_client = cacher_client.clone();
        move || AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone(), stream_producer.clone(), updater_config)
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
        .jobs(WorkerJob::UpdatePerpetuals, PerpetualUpdater::chains(), |chain, _| {
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
        })
        .job(WorkerJob::UpdateUsageRanks, {
            let database = database.clone();
            move || {
                let updater = UsageRankUpdater::new(database.clone());
                async move { updater.update_usage_ranks().await }
            }
        })
        .jobs(WorkerJob::UpdateAssetsImages, Chain::all(), |chain, _| {
            let static_assets_client = StaticAssetsClient::new(&settings.assets.url);
            let database = database.clone();
            move || {
                let updater = AssetsImagesUpdater::new(static_assets_client.clone(), database.clone());
                async move { updater.update_chain(chain).await }
            }
        })
        .jobs(WorkerJob::UpdateStakingApy, Chain::stakeable(), |chain, _| {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let database = database.clone();
                async move {
                    let providers = ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("staking_apy")));
                    let updater = StakeApyUpdater::new(providers, database.clone());
                    updater.update_chain(chain).await
                }
            }
        })
        .jobs(WorkerJob::UpdateChainValidators, Chain::stakeable(), |chain, _| {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let database = database.clone();
                async move {
                    let providers = Arc::new(ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_validators"))));
                    let scanner = ValidatorScanner::new(providers, database);
                    scanner.update_validators_for_chain(chain).await
                }
            }
        })
        .jobs(WorkerJob::UpdateValidatorsFromStaticAssets, [Chain::Tron, Chain::SmartChain], |chain, _| {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let database = database.clone();
                async move {
                    let providers = Arc::new(ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_static_assets"))));
                    let assets_url = settings.assets.url.clone();
                    let scanner = ValidatorScanner::new(providers, database);
                    scanner.update_validators_from_static_assets_for_chain(chain, &assets_url).await
                }
            }
        })
        .finish()
}
