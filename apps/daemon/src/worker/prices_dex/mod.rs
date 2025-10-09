mod prices_dex_updater;

use prices_dex_updater::PricesDexUpdater;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;

pub async fn jobs(_settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let updater = PricesDexUpdater::new("https://hermes.pyth.network", "https://lite-api.jup.ag");
    updater.update_prices().await.ok();
    updater.update_real_time_price().await.ok();

    vec![]
}
