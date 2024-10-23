extern crate rocket;
use pricer::price_client::PriceClient;
use primitives::asset_price::{AssetPrices, AssetPricesRequest};
use primitives::{AssetMarketPrice, DEFAULT_FIAT_CURRENCY};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_price(asset_id: &str, currency: Option<&str>, price_client: &State<Mutex<PriceClient>>) -> Json<AssetMarketPrice> {
    let currency = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);

    let price = price_client.lock().await.get_asset_price(asset_id, currency).await.unwrap();

    Json(price)
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(request: Json<AssetPricesRequest>, price_client: &State<Mutex<PriceClient>>) -> Json<AssetPrices> {
    let currency: String = request.currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());

    let asset_ids = request.asset_ids.iter().map(|x| x.as_str()).collect();
    let prices = price_client.lock().await.get_asset_prices(currency.as_str(), asset_ids).await.unwrap();

    Json(prices)
}
