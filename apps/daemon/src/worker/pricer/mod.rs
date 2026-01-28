mod charts_updater;
mod markets_updater;
mod observed_prices_updater;
pub mod price_updater;

use std::error::Error;
use std::sync::Arc;

use cacher::CacherClient;
use charts_updater::ChartsUpdater;
use coingecko::CoinGeckoClient;
use job_runner::{ShutdownReceiver, run_job};
use markets_updater::MarketsUpdater;
use observed_prices_updater::ObservedPricesUpdater;
use price_updater::{PriceUpdater, UpdatePrices};
use pricer::{MarketsClient, PriceClient};
use primitives::ConfigKey;
use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::StreamProducer;
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let price_outdated = config.get_duration(ConfigKey::PriceOutdated)?.as_secs();

    let clean_updated_assets = tokio::spawn(run_job(
        "Clean outdated assets",
        config.get_duration(ConfigKey::PriceTimerCleanOutdated)?,
        shutdown_rx.clone(),
        {
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            let settings = Arc::new(settings.clone());
            move || {
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                let settings = Arc::clone(&settings);
                async move {
                    let updater = price_updater_factory(&database, &cacher_client, &settings).await;
                    updater.clean_outdated_assets(price_outdated).await
                }
            }
        },
    ));
    let update_fiat_assets = tokio::spawn(run_job("Update fiat assets", config.get_duration(ConfigKey::PriceTimerFiatRates)?, shutdown_rx.clone(), {
        let settings = Arc::new(settings.clone());
        let cacher_client = cacher_client.clone();
        let database = database.clone();
        move || {
            let settings = Arc::clone(&settings);
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            async move {
                let updater = price_updater_factory(&database, &cacher_client, &settings).await;
                updater.update_fiat_rates().await
            }
        }
    }));

    let update_prices_top_market_cap = tokio::spawn(run_job(
        "Update prices top (top 500) market cap",
        config.get_duration(ConfigKey::PriceTimerTopMarketCap)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = price_updater_factory(&database, &cacher_client, &settings.clone()).await;
                    updater.update_prices_type(UpdatePrices::Top).await
                }
            }
        },
    ));

    let update_prices_high_market_cap = tokio::spawn(run_job(
        "Update prices high (500-2500) market cap",
        config.get_duration(ConfigKey::PriceTimerHighMarketCap)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = price_updater_factory(&database, &cacher_client, &settings.clone()).await;
                    updater.update_prices_type(UpdatePrices::High).await
                }
            }
        },
    ));

    let update_prices_low_market_cap = tokio::spawn(run_job(
        "Update prices low (2500...) market cap",
        config.get_duration(ConfigKey::PriceTimerLowMarketCap)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = price_updater_factory(&database, &cacher_client, &settings.clone()).await;
                    updater.update_prices_type(UpdatePrices::Low).await
                }
            }
        },
    ));

    let update_hourly_charts_job = tokio::spawn(run_job(
        "Aggregate hourly charts",
        config.get_duration(ConfigKey::PriceTimerChartsHourly)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = charts_updater_factory(&database, &cacher_client, &settings, coingecko_client).await;
                    updater.aggregate_hourly_charts().await
                }
            }
        },
    ));

    let update_daily_charts_job = tokio::spawn(run_job(
        "Aggregate daily charts",
        config.get_duration(ConfigKey::PriceTimerChartsDaily)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = charts_updater_factory(&database, &cacher_client, &settings, coingecko_client).await;
                    updater.aggregate_daily_charts().await
                }
            }
        },
    ));

    let cleanup_charts_data_job = tokio::spawn(run_job(
        "Cleanup charts data",
        config.get_duration(ConfigKey::PriceTimerCleanupCharts)?,
        shutdown_rx.clone(),
        {
            let settings = Arc::new(settings.clone());
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let updater = charts_updater_factory(&database, &cacher_client, &settings, coingecko_client).await;
                    updater.cleanup_charts_data().await
                }
            }
        },
    ));

    let update_markets = tokio::spawn(run_job("Update markets", config.get_duration(ConfigKey::PriceTimerMarkets)?, shutdown_rx.clone(), {
        let settings = Arc::new(settings.clone());
        let cacher_client = cacher_client.clone();
        let database = database.clone();
        move || {
            let settings = Arc::clone(&settings);
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            async move { markets_updater_factory(&database, &cacher_client, &settings.clone()).update_markets().await }
        }
    }));

    let max_observed_assets = config.get_usize(ConfigKey::PriceObservedMaxAssets)?;
    let min_observers = config.get_usize(ConfigKey::PriceObservedMinObservers)?;
    let update_observed_prices = tokio::spawn(run_job(
        "Update observed prices",
        config.get_duration(ConfigKey::PriceObservedFetchInterval)?,
        shutdown_rx,
        {
            let settings = Arc::new(settings.clone());
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = Arc::clone(&settings);
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    let price_client = PriceClient::new(database.clone(), cacher_client.clone());
                    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "observed_prices_worker")
                        .await
                        .expect("Failed to create stream producer");
                    let updater = ObservedPricesUpdater::new(cacher_client, price_client, stream_producer, max_observed_assets, min_observers);
                    updater.update().await
                }
            }
        },
    ));

    Ok(vec![
        clean_updated_assets,
        update_fiat_assets,
        update_prices_top_market_cap,
        update_prices_high_market_cap,
        update_prices_low_market_cap,
        update_hourly_charts_job,
        update_daily_charts_job,
        cleanup_charts_data_job,
        update_markets,
        update_observed_prices,
    ])
}

async fn price_updater_factory(database: &Database, cacher: &CacherClient, settings: &Settings) -> PriceUpdater {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let price_client = PriceClient::new(database.clone(), cacher.clone());
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "pricer_worker")
        .await
        .expect("Failed to create stream producer");
    PriceUpdater::new(price_client, coingecko_client, stream_producer)
}

async fn charts_updater_factory(database: &Database, cacher: &CacherClient, settings: &Settings, coingecko_client: CoinGeckoClient) -> ChartsUpdater {
    let price_client = PriceClient::new(database.clone(), cacher.clone());
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "charts_worker")
        .await
        .expect("Failed to create stream producer");
    ChartsUpdater::new(price_client, coingecko_client, stream_producer)
}

fn markets_updater_factory(database: &Database, cacher: &CacherClient, settings: &Settings) -> MarketsUpdater {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let markets_client = MarketsClient::new(database.clone(), cacher.clone());
    MarketsUpdater::new(markets_client, coingecko_client)
}
