mod asset_rank_updater;
pub mod asset_updater;
mod assets_images_updater;
mod perpetual_updater;
mod staking_apy_updater;
mod usage_rank_updater;

use api_connector::StaticAssetsClient;
use asset_rank_updater::AssetRankUpdater;
use asset_updater::AssetUpdater;
use assets_images_updater::AssetsImagesUpdater;
use cacher::CacherClient;
use coingecko::CoinGeckoClient;
use job_runner::{ShutdownReceiver, run_job};
use perpetual_updater::PerpetualUpdater;
use primitives::ConfigKey;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use std::error::Error;
use storage::ConfigCacher;
use tokio::task::JoinHandle;
use usage_rank_updater::UsageRankUpdater;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = ConfigCacher::new(database.clone());

    let update_existing_assets = tokio::spawn(run_job(
        "Update existing prices assets",
        config.get_duration(ConfigKey::AssetsTimerUpdateExisting)?,
        shutdown_rx.clone(),
        {
            let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
            move || {
                let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
                async move { asset_updater.update_existing_assets().await }
            }
        },
    ));

    let update_all_assets = tokio::spawn(run_job(
        "Update all prices assets",
        config.get_duration(ConfigKey::AssetsTimerUpdateAll)?,
        shutdown_rx.clone(),
        {
            let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
            move || {
                let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
                async move { asset_updater.update_assets().await }
            }
        },
    ));

    let update_native_prices_assets = tokio::spawn(run_job(
        "Update native prices assets",
        config.get_duration(ConfigKey::AssetsTimerUpdateNative)?,
        shutdown_rx.clone(),
        {
            let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
            move || {
                let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
                async move { asset_updater.update_native_prices_assets().await }
            }
        },
    ));

    let update_tranding_assets = tokio::spawn(run_job(
        "Update CoinGecko Trending assets",
        config.get_duration(ConfigKey::AssetsTimerUpdateTrending)?,
        shutdown_rx.clone(),
        {
            let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
            move || {
                let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
                async move { asset_updater.update_trending_assets().await }
            }
        },
    ));

    let update_recently_added_assets = tokio::spawn(run_job(
        "Update CoinGecko recently added assets",
        config.get_duration(ConfigKey::AssetsTimerUpdateRecentlyAdded)?,
        shutdown_rx.clone(),
        {
            let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
            move || {
                let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
                async move { asset_updater.update_recently_added_assets().await }
            }
        },
    ));

    let update_suspicious_assets = tokio::spawn(run_job(
        "Update suspicious asset ranks",
        config.get_duration(ConfigKey::AssetsTimerUpdateSuspicious)?,
        shutdown_rx.clone(),
        {
            let database = database.clone();
            move || {
                let suspicious_updater = AssetRankUpdater::new(database.clone());
                async move { suspicious_updater.update_suspicious_assets().await }
            }
        },
    ));

    let update_staking_apy = tokio::spawn(run_job(
        "Update staking APY",
        config.get_duration(ConfigKey::AssetsTimerUpdateStakingApy)?,
        shutdown_rx.clone(),
        {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let chain_providers = ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("staking_apy")));
                let updater = StakeApyUpdater::new(chain_providers, database.clone());
                async move { updater.update_staking_apy().await }
            }
        },
    ));

    let update_perpetuals = tokio::spawn(run_job(
        "Update perpetuals",
        config.get_duration(ConfigKey::AssetsTimerUpdatePerpetuals)?,
        shutdown_rx.clone(),
        {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let updater = PerpetualUpdater::new(settings.clone(), database.clone());
                async move { updater.update_perpetuals().await }
            }
        },
    ));

    let update_usage_ranks = tokio::spawn(run_job(
        "Update usage ranks",
        config.get_duration(ConfigKey::AssetsTimerUpdateUsageRank)?,
        shutdown_rx.clone(),
        {
            let database = database.clone();
            move || {
                let updater = UsageRankUpdater::new(database.clone());
                async move { updater.update_usage_ranks().await }
            }
        },
    ));

    let update_assets_images = tokio::spawn(run_job("Update assets images", config.get_duration(ConfigKey::AssetsTimerUpdateImages)?, shutdown_rx, {
        let static_assets_client = StaticAssetsClient::new(&settings.assets.url);
        let database = database.clone();
        move || {
            let updater = AssetsImagesUpdater::new(static_assets_client.clone(), database.clone());
            async move { updater.update_assets_images().await }
        }
    }));

    Ok(vec![
        update_existing_assets,
        update_all_assets,
        update_native_prices_assets,
        update_tranding_assets,
        update_recently_added_assets,
        update_suspicious_assets,
        update_staking_apy,
        update_perpetuals,
        update_usage_ranks,
        update_assets_images,
    ])
}
