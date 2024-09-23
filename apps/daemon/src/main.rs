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
use job_runner::run_job;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    println!("daemon init");
    let settings = settings::Settings::new().unwrap();

    let update_fiat_assets = run_job("update fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_fiat_assets().await }
        }
    });

    let update_appstore_version = run_job("update app store version", Duration::from_secs(43200), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut version_client = VersionClient::new(&settings.postgres.url);
            async move { version_client.update_ios_version().await }
        }
    });

    // let update_apk_version = run_job("update apk version", Duration::from_secs(43200), {
    //     let settings = Arc::new(settings.clone());
    //     move || {
    //         let mut version_client = VersionClient::new(&settings.postgres.url);
    //         async move { version_client.update_apk_version().await }
    //     }
    // });

    let update_oneinch_tokenlist = run_job("update 1inch token list", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut oneinch_updater = oneinch_updater::Client::new(
                swap_oneinch::OneInchClient::new(&settings.swap.oneinch.url, &settings.swap.oneinch.key, 0.0, "".to_string()),
                &settings.postgres.url,
            );
            async move { oneinch_updater.update_swap_tokenlist().await }
        }
    });

    let device_updater = run_job("device updater", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut device_updater = DeviceUpdater::new(&settings.postgres.url);
            async move { device_updater.update().await }
        }
    });

    let token_list_updater = run_job("token list update", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let assets_client = AssetsClient::new(&settings.assets.url);
            let mut tokenlist_client = TokenListClient::new(&settings.postgres.url, assets_client);
            async move { tokenlist_client.update().await }
        }
    });

    let _ = tokio::join!(
        update_fiat_assets,
        update_appstore_version,
        //update_apk_version,
        update_oneinch_tokenlist,
        device_updater,
        token_list_updater
    );
}
