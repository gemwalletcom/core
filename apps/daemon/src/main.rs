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
use std::thread;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    println!("daemon init");
    let settings = settings::Settings::new().unwrap();
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

    loop {
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
