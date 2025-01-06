mod asset_updater;

use asset_updater::AssetUpdater;
use coingecko::CoinGeckoClient;
use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);

    let update_assets = run_job("Update CoinGecko assets", Duration::from_secs(86400), {
        let settings = settings.clone();
        let coingecko_client = coingecko_client.clone();
        move || {
            let mut asset_updater = AssetUpdater::new(coingecko_client.clone(), &settings.postgres.url);
            async move { asset_updater.update_assets().await }
        }
    });

    vec![Box::pin(update_assets)]
}
