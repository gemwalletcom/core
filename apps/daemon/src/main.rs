mod device_updater;
mod fiat_assets_updater;
mod tokenlist_updater;
mod version_updater;

use crate::device_updater::DeviceUpdater;
use crate::tokenlist_updater::Client as TokenListClient;
use crate::version_updater::Client as VersionClient;

use api_connector::AssetsClient;
use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use pricer::client::PriceClient;
use pricer::coingecko::CoinGeckoClient;
use pricer::price_updater::PriceUpdater;
use std::thread;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    println!("daemon init");
    let settings = settings::Settings::new().unwrap();
    let price_client = PriceClient::new(
        &settings.redis.url,
        &settings.postgres.url,
        &settings.clickhouse.url,
    )
    .await
    .unwrap();
    let coingecko_client = CoinGeckoClient::new(settings.coingecko.key.secret);
    let mut price_updater = PriceUpdater::new(price_client, coingecko_client);
    let mut version_client = VersionClient::new(&settings.postgres.url);
    let mut device_updater = DeviceUpdater::new(&settings.postgres.url);
    let assets_client = AssetsClient::new(settings.assets.url);
    let mut tokenlist_client = TokenListClient::new(&settings.postgres.url, &assets_client);

    let providers = FiatProviderFactory::new_providers(settings::Settings::new().unwrap());
    let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);

    // update fiat assets
    let result = fiat_assets_updater.update_fiat_assets().await;
    match result {
        Ok(count) => {
            println!("update fiat assets: {}", count)
        }
        Err(err) => {
            println!("update fiat assets error: {}", err)
        }
    }

    // update assets
    let result = price_updater.update_assets().await;
    match result {
        Ok(count) => {
            println!("update assets: {}", count)
        }
        Err(err) => {
            println!("update assets error: {}", err)
        }
    }

    // update rates
    let result = price_updater.update_fiat_rates().await;
    match result {
        Ok(count) => {
            println!("update rates: {}", count)
        }
        Err(err) => {
            println!("update rates error: {}", err)
        }
    }

    // update version
    let ios_version = version_client.update_ios_version().await;
    match ios_version {
        Ok(version) => {
            println!("ios version: {:?}", version)
        }
        Err(err) => {
            println!("ios version error: {}", err)
        }
    }

    // update device
    let result = device_updater.update().await;
    match result {
        Ok(result) => {
            println!("device updater result: {:?}", result)
        }
        Err(err) => {
            println!("device updater result error: {}", err)
        }
    }

    // updates charts
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
        // updates prices
        let result = price_updater.update_prices().await;
        match result {
            Ok(count) => {
                println!("update prices: {}", count)
            }
            Err(err) => {
                println!("update prices error: {}", err)
            }
        }

        // update cache
        let result = price_updater.update_cache().await;
        match result {
            Ok(count) => {
                println!("update prices cache: {}", count)
            }
            Err(err) => {
                println!("update prices cache error: {}", err)
            }
        }

        let result = tokenlist_client.update().await;
        match result {
            Ok(count) => {
                println!("update tokenlist versions: {}", count)
            }
            Err(err) => {
                println!("update tokenlist versions error: {}", err)
            }
        }

        thread::sleep(Duration::from_secs(settings.pricer.timer));
    }
}
