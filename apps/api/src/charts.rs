extern crate rocket;
use pricer::chart_client::ChartClient;
use pricer::PriceClient;
use primitives::asset_price::{ChartPeriod, Charts};
use primitives::DEFAULT_FIAT_CURRENCY;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/charts/<asset_id>?<period>&<currency>")]
pub async fn get_charts(
    asset_id: String,
    period: Option<String>,
    currency: Option<String>,
    charts_client: &State<Mutex<ChartClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Json<Charts> {
    let period = ChartPeriod::new(period.unwrap_or_default()).unwrap_or(ChartPeriod::Day);
    let currency_value = currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());

    let coin_id = charts_client.lock().await.get_coin_id(asset_id.as_str()).unwrap();

    let prices = charts_client
        .lock()
        .await
        .get_charts_prices(coin_id.as_str(), period, currency_value.as_str())
        .await
        .unwrap();

    let asset_price = price_client
        .lock()
        .await
        .get_asset_price(asset_id.as_str(), currency_value.as_str())
        .await
        .unwrap();

    let response = Charts {
        price: asset_price.price,
        market: asset_price.market,
        prices,
        market_caps: vec![],
        total_volumes: vec![],
    };

    Json(response)
}
