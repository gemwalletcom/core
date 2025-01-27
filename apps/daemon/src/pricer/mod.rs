mod charts_updater;
mod price_asset_updater;
mod price_updater;

use crate::pricer::charts_updater::ChartsUpdater;
use crate::pricer::price_updater::PriceUpdater;
use coingecko::CoinGeckoClient;
use job_runner::run_job;
use price_asset_updater::PriceAssetUpdater;
use price_updater::UpdatePrices;
use pricer::{ChartClient, PriceClient};
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use storage::ClickhouseClient;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);

    let clean_updated_assets = run_job("Clean outdated assets", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings).clean_outdated_assets(settings.pricer.outdated).await }
        }
    });
    let update_fiat_assets = run_job("Update fiat assets", Duration::from_secs(360), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_fiat_rates().await }
        }
    });

    let update_prices_assets = run_job("Update prices assets", Duration::from_secs(1800), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_asset_updater_factory(&settings.clone()).update_prices_assets().await }
        }
    });

    let update_prices_assets_pages = run_job("Update prices assets 30 pages", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_prices_pages(30).await }
        }
    });

    let update_prices_top_market_cap = run_job("Update prices top (top 500) market cap", Duration::from_secs(settings.pricer.timer), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_prices_type(UpdatePrices::Top).await }
        }
    });

    let update_prices_high_market_cap = run_job("Update prices high (500-2500) market cap", Duration::from_secs(settings.pricer.timer * 3), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_prices_type(UpdatePrices::High).await }
        }
    });

    let update_prices_low_market_cap = run_job("Update prices low (2500...) market cap", Duration::from_secs(settings.pricer.timer * 10), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_prices_type(UpdatePrices::Low).await }
        }
    });

    let update_prices_cache = run_job("Update prices cache", Duration::from_secs(30), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            async move { price_updater_factory(&settings.clone()).update_prices_cache().await }
        }
    });

    let update_charts = run_job("Update charts", Duration::from_secs(settings.charter.timer), {
        let settings = settings.clone();
        let coingecko_client = coingecko_client.clone();
        move || {
            let clickhouse_database = ClickhouseClient::new(&settings.clickhouse.url, &settings.clickhouse.database);
            let charts_client = ChartClient::new(&settings.postgres.url, clickhouse_database);
            let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url);
            let mut charts_updater = ChartsUpdater::new(charts_client, price_client, coingecko_client.clone());
            async move { charts_updater.update_charts().await }
        }
    });

    vec![
        Box::pin(clean_updated_assets),
        Box::pin(update_fiat_assets),
        Box::pin(update_prices_assets),
        Box::pin(update_prices_assets_pages),
        Box::pin(update_prices_top_market_cap),
        Box::pin(update_prices_high_market_cap),
        Box::pin(update_prices_low_market_cap),
        Box::pin(update_prices_cache),
        Box::pin(update_charts),
    ]
}

fn price_updater_factory(settings: &Settings) -> PriceUpdater {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url.clone());
    PriceUpdater::new(price_client, coingecko_client.clone())
}

fn price_asset_updater_factory(settings: &Settings) -> PriceAssetUpdater {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url.clone());
    PriceAssetUpdater::new(price_client, coingecko_client.clone())
}
