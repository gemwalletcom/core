use charter::{charts_updater::ChartsUpdater, client::ChartsClient};
use coingecko::CoinGeckoClient;
use job_runner::run_job;
use pricer::client::PriceClient;
use settings::Settings;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    println!("charter init");

    let settings = Settings::new().unwrap();

    let update_charts = run_job("Update charts", Duration::from_secs(settings.charter.timer), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
            let charts_client = ChartsClient::new(&settings.postgres.url, &settings.clickhouse.url);
            let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url);
            let mut charts_updater = ChartsUpdater::new(charts_client, price_client, coingecko_client);
            async move { charts_updater.update_charts().await }
        }
    });

    let _ = tokio::join!(update_charts);
}
