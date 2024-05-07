use assetor::AssetUpdater;
use coingecko::CoinGeckoClient;
use settings::Settings;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("assetor init");

    let settings = Settings::new().unwrap();
    let coingecko_client = CoinGeckoClient::new(settings.coingecko.key.secret);
    let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), &settings.postgres.url);

    loop {
        println!("update assets: start");

        match asset_updater.update_assets().await {
            Ok(count) => {
                println!("update assets: {}", count)
            }
            Err(err) => {
                println!("update assets error: {}", err)
            }
        }

        thread::sleep(Duration::from_secs(81600));
    }
}
