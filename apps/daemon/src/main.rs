mod device_updater;
mod fiat_assets_updater;
mod oneinch_updater;
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

    let mut oneinch_updater = oneinch_updater::Client::new(
        swap_oneinch::OneInchClient::new(
            settings.swap.oneinch.url,
            settings.swap.oneinch.key,
            0.0,
            "".to_string(),
        ),
        &settings.postgres.url,
    );

    // update fiat assets
    match fiat_assets_updater.update_fiat_assets().await {
        Ok(count) => {
            println!("update fiat assets: {}", count)
        }
        Err(err) => {
            println!("update fiat assets error: {}", err)
        }
    }

    // update oneinch swap tokenlist

    match oneinch_updater.update_swap_tokenlist().await {
        Ok(_) => {
            println!("update oneinch swap tokenlist: complete")
        }
        Err(err) => {
            println!("update oneinch swap tokenlist error: {}", err)
        }
    }
    // update version
    match version_client.update_ios_version().await {
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
