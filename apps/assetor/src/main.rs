use assetor::AssetUpdater;
use coingecko::CoinGeckoClient;
use job_runner::run_job;
use settings::Settings;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    println!("assetor init");

    let settings = Settings::new().unwrap();

    let update_assets = run_job("Update assets assets", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings = Arc::clone(&settings);
            let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
            let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), &settings.postgres.url);
            async move { asset_updater.update_assets().await }
        }
    });

    let _ = tokio::join!(update_assets);
}
