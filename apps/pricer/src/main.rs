use coingecko::CoinGeckoClient;
use pricer::{client::PriceClient, price_updater::PriceUpdater};
use settings::Settings;
use std::{error::Error, thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("pricer init");

    let settings = Settings::new().unwrap();
    let coingecko_client = CoinGeckoClient::new(settings.coingecko.key.secret);
    let price_client = PriceClient::new(
        &settings.redis.url,
        &settings.postgres.url,
        &settings.clickhouse.url,
    );

    let mut price_updater: PriceUpdater = PriceUpdater::new(price_client, coingecko_client);

    println!("update assets: start");
    log_method_call("update assets", price_updater.update_assets().await);

    println!("update rates: start");
    log_method_call("update rates", price_updater.update_fiat_rates().await);

    // updates charts， only needed on initial setup
    // log_method_call("update charts", price_updater.update_charts().await);

    loop {
        println!("update prices: start");

        log_method_call("update prices", price_updater.update_prices().await);

        println!("update prices cache: start");
        log_method_call("update prices cache", price_updater.update_cache().await);

        thread::sleep(Duration::from_secs(settings.pricer.timer));
    }
}

fn log_method_call(method: &str, result: Result<usize, Box<dyn Error>>) {
    match result {
        Ok(count) => {
            println!("{}: {}", method, count)
        }
        Err(err) => {
            println!("{} error: {}", method, err)
        }
    }
}
