mod asset_rank_updater;
mod asset_updater;
mod perpetual_updater;
mod staking_apy_updater;

use asset_rank_updater::AssetRankUpdater;
use asset_updater::AssetUpdater;
use coingecko::CoinGeckoClient;
use job_runner::run_job;
use perpetual_updater::PerpetualUpdater;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use staking_apy_updater::StakeApyUpdater;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);

    let update_assets = run_job("Update CoinGecko assets", Duration::from_secs(86400), {
        let (coingecko_client, database) = (coingecko_client.clone(), database.clone());
        move || {
            let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone());
            async move { asset_updater.update_assets().await }
        }
    });

    let update_tranding_assets = run_job("Update CoinGecko Trending assets", Duration::from_secs(3600), {
        let (coingecko_client, database) = (coingecko_client.clone(), database.clone());
        move || {
            let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone());
            async move { asset_updater.update_trending_assets().await }
        }
    });

    let update_recently_added_assets = run_job("Update CoinGecko recently added assets", Duration::from_secs(3600), {
        let (coingecko_client, database) = (coingecko_client.clone(), database.clone());
        move || {
            let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), database.clone());
            async move { asset_updater.update_recently_added_assets().await }
        }
    });

    let update_suspicious_assets = run_job("Update suspicious asset ranks", Duration::from_secs(3600), {
        let database = database.clone();
        move || {
            let mut suspicious_updater = AssetRankUpdater::new(database.clone());
            async move { suspicious_updater.update_suspicious_assets().await }
        }
    });

    let update_staking_apy = run_job("Update staking APY", Duration::from_secs(86400), {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let chain_providers = ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("staking_apy")));
            let mut updater = StakeApyUpdater::new(chain_providers, database.clone());
            async move { updater.update_staking_apy().await }
        }
    });

    let update_perpetuals = run_job("Update perpetuals", Duration::from_secs(3600), {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let mut updater = PerpetualUpdater::new(settings.clone(), database.clone());
            async move { updater.update_perpetuals().await }
        }
    });

    vec![
        Box::pin(update_assets),
        Box::pin(update_tranding_assets),
        Box::pin(update_recently_added_assets),
        Box::pin(update_suspicious_assets),
        Box::pin(update_staking_apy),
        Box::pin(update_perpetuals),
    ]
}
