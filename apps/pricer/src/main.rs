use coingecko::CoinGeckoClient;
use pricer::{client::PriceClient, price_updater::PriceUpdater};
use settings::Settings;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("pricer init");

    let settings = Settings::new().unwrap();
    let coingecko_client = CoinGeckoClient::new(settings.coingecko.key.secret);
    let price_client = PriceClient::new(&settings.redis.url, &settings.postgres.url);

    let mut price_updater = PriceUpdater::new(price_client, coingecko_client.clone());

    println!("clean outdated asset: start");

    match price_updater
        .clean_outdated_assets(settings.pricer.outdated)
        .await
    {
        Ok(count) => {
            println!("clean outdated assets: {},", count)
        }
        Err(err) => {
            println!("clean outdated assets error: {}", err)
        }
    }

    println!("update rates: start");

    match price_updater.update_fiat_rates().await {
        Ok(count) => {
            println!("update rates: {}", count)
        }
        Err(err) => {
            println!("update rates error: {}", err)
        }
    }

    println!("update prices assets: start");

    match price_updater.update_prices_assets().await {
        Ok(count) => {
            println!("update prices assets: {}", count)
        }
        Err(err) => {
            println!("update prices assets error: {}", err)
        }
    }

    loop {
        println!("update prices: start");

        match price_updater.update_prices(25).await {
            Ok(count) => {
                println!("update prices: {}", count)
            }
            Err(err) => {
                println!("update prices error: {}", err)
            }
        }

        println!("update prices cache: start");

        match price_updater.update_prices_cache().await {
            Ok(count) => {
                println!("update prices cache: {}", count)
            }
            Err(err) => {
                println!("update prices cache error: {}", err)
            }
        }

        thread::sleep(Duration::from_secs(settings.pricer.timer));
    }
}
