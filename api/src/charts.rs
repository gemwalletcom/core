extern crate rocket;
use pricer::DEFAULT_FIAT_CURRENCY;
use rocket::serde::json::Json;
use pricer::client::Client as PriceClient;
use rocket::State;
use rocket::tokio::sync::Mutex;
use primitives::asset_price::{Charts, ChartPeriod};

#[get("/charts/<asset_id>?<period>&<currency>")]
pub async fn get_charts(asset_id: String, period: Option<String>, currency: Option<String>, price_client: &State<Mutex<PriceClient>>) -> Json<Charts> {
    let period = ChartPeriod::new(period.unwrap_or_default()).unwrap_or(ChartPeriod::Day);
    let _currency: String = currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());    
    let coin_id =  price_client.lock().await.get_coin_id(asset_id.as_str()).unwrap();
    let prices = price_client.lock().await.get_charts_prices(coin_id.as_str(), &period).unwrap();
    let response = Charts{
        prices,
        market_caps: vec![],
        total_volumes: vec![],
    };

    Json(response)
}
