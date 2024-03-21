use coingecko::CoinGeckoClient;
use pricer::{client::PriceClient, price_updater::PriceUpdater};
use settings::Settings;
use std::{thread, time::Duration};

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

    let mut price_updater = PriceUpdater::new(price_client, coingecko_client.clone());
    //let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), &settings.postgres.url);

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

    //println!("update assets: start");
    // match asset_updater.update_assets().await {
    //     Ok(count) => {
    //         println!("update assets: {}", count)
    //     }
    //     Err(err) => {
    //         println!("update assets error: {}", err)
    //     }
    // }

    println!("update rates: start");

    match price_updater.update_fiat_rates().await {
        Ok(count) => {
            println!("update rates: {}", count)
        }
        Err(err) => {
            println!("update rates error: {}", err)
        }
    }

    // updates charts
    // only needed on initial setup
    // let result = price_updater.update_charts().await;
    // match result {
    //     Ok(count) => {
    //         println!("update charts: {}", count)
    //     }
    //     Err(err) => {
    //         println!("update charts error: {}", err)
    //     }
    // }

    loop {
        println!("update prices: start");

        match price_updater.update_prices().await {
            Ok(count) => {
                println!("update prices: {}", count)
            }
            Err(err) => {
                println!("update prices error: {}", err)
            }
        }

        println!("update prices cache: start");

        match price_updater.update_cache().await {
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
