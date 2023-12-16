extern crate rocket;
use pricer::client::Client as PriceClient;
use pricer::DEFAULT_FIAT_CURRENCY;
use primitives::asset_price::{ChartPeriod, Charts};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/charts/<asset_id>?<period>&<currency>")]
pub async fn get_charts(
    asset_id: String,
    period: Option<String>,
    currency: Option<String>,
    price_client: &State<Mutex<PriceClient>>,
) -> Json<Charts> {
    let period = ChartPeriod::new(period.unwrap_or_default()).unwrap_or(ChartPeriod::Day);
    let currency_value = currency
        .clone()
        .unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());
    let coin_id = price_client
        .lock()
        .await
        .get_coin_id(asset_id.as_str())
        .unwrap();
    let prices = price_client
        .lock()
        .await
        .get_charts_prices(coin_id.as_str(), period, currency_value.as_str())
        .await
        .unwrap();
    let response = Charts {
        prices,
        market_caps: vec![],
        total_volumes: vec![],
    };

    Json(response)
}
