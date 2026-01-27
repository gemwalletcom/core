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
use job_runner::run_job;
use perpetual_updater::PerpetualUpdater;
use primitives::ConfigKey;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use storage::ConfigCacher;
use usage_rank_updater::UsageRankUpdater;

pub async fn jobs(settings: Settings) -> Result<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = ConfigCacher::new(database.clone());

    let update_existing_assets = run_job("Update existing prices assets", config.get_duration(ConfigKey::AssetsTimerUpdateExisting)?, {
        let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
        move || {
            let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
            async move { asset_updater.update_existing_assets().await }
        }
    });

    let update_all_assets = run_job("Update all prices assets", config.get_duration(ConfigKey::AssetsTimerUpdateAll)?, {
        let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
        move || {
            let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
            async move { asset_updater.update_assets().await }
        }
    });

    let update_native_prices_assets = run_job("Update native prices assets", config.get_duration(ConfigKey::AssetsTimerUpdateNative)?, {
        let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
        move || {
            let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
            async move { asset_updater.update_native_prices_assets().await }
        }
    });

    let update_tranding_assets = run_job("Update CoinGecko Trending assets", config.get_duration(ConfigKey::AssetsTimerUpdateTrending)?, {
        let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
        move || {
            let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
            async move { asset_updater.update_trending_assets().await }
        }
    });

    let update_recently_added_assets = run_job("Update CoinGecko recently added assets", config.get_duration(ConfigKey::AssetsTimerUpdateRecentlyAdded)?, {
        let (coingecko_client, database, cacher_client) = (coingecko_client.clone(), database.clone(), cacher_client.clone());
        move || {
            let asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone(), cacher_client.clone());
            async move { asset_updater.update_recently_added_assets().await }
        }
    });

    let update_suspicious_assets = run_job("Update suspicious asset ranks", config.get_duration(ConfigKey::AssetsTimerUpdateSuspicious)?, {
        let database = database.clone();
        move || {
            let suspicious_updater = AssetRankUpdater::new(database.clone());
            async move { suspicious_updater.update_suspicious_assets().await }
        }
    });

    let update_staking_apy = run_job("Update staking APY", config.get_duration(ConfigKey::AssetsTimerUpdateStakingApy)?, {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let chain_providers = ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("staking_apy")));
            let updater = StakeApyUpdater::new(chain_providers, database.clone());
            async move { updater.update_staking_apy().await }
        }
    });

    let update_perpetuals = run_job("Update perpetuals", config.get_duration(ConfigKey::AssetsTimerUpdatePerpetuals)?, {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let updater = PerpetualUpdater::new(settings.clone(), database.clone());
            async move { updater.update_perpetuals().await }
        }
    });

    let update_usage_ranks = run_job("Update usage ranks", config.get_duration(ConfigKey::AssetsTimerUpdateUsageRank)?, {
        let database = database.clone();
        move || {
            let updater = UsageRankUpdater::new(database.clone());
            async move { updater.update_usage_ranks().await }
        }
    });

    let update_assets_images = run_job("Update assets images", config.get_duration(ConfigKey::AssetsTimerUpdateImages)?, {
        let static_assets_client = StaticAssetsClient::new(&settings.assets.url);
        let database = database.clone();
        move || {
            let updater = AssetsImagesUpdater::new(static_assets_client.clone(), database.clone());
            async move { updater.update_assets_images().await }
        }
    });

    Ok(vec![
        Box::pin(update_existing_assets),
        Box::pin(update_all_assets),
        Box::pin(update_native_prices_assets),
        Box::pin(update_tranding_assets),
        Box::pin(update_recently_added_assets),
        Box::pin(update_suspicious_assets),
        Box::pin(update_staking_apy),
        Box::pin(update_perpetuals),
        Box::pin(update_usage_ranks),
        Box::pin(update_assets_images),
    ])
}
