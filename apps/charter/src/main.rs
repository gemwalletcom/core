use charter::{charts_updater::ChartsUpdater, client::ChartsClient};
use coingecko::CoinGeckoClient;
use pricer::client::PriceClient;
use settings::Settings;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("charter init");

    let settings = Settings::new().unwrap();
    let coingecko_client = CoinGeckoClient::new(settings.coingecko.key.secret);
    let charts_client = ChartsClient::new(&settings.postgres.url, &settings.clickhouse.url);
    let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url);
    let mut charts_updater = ChartsUpdater::new(charts_client, price_client, coingecko_client);

    // updates charts
    // only needed on initial setup
    // println!("update charts: start");
    // let result = charts_updater.update_charts_all().await;
    // match result {
    //     Ok(count) => {
    //         println!("update charts all: {}", count)
    //     }
    //     Err(err) => {
    //         println!("update charts all error: {}", err)
    //     }
    // }

    loop {
        println!("update charts: start");

        //TODO: In the future pricer should push events into the queue and consumed by the charter
        match charts_updater.update_charts().await {
            Ok(count) => {
                println!("update charts: {}", count)
            }
            Err(err) => {
                println!("update charts error: {}", err)
            }
        }

        thread::sleep(Duration::from_secs(settings.charter.timer));
    }
}
