extern crate rocket;
use pricer::client::PriceClient;
use pricer::DEFAULT_FIAT_CURRENCY;
use primitives::asset_price::{AssetPrice, AssetPrices, AssetPricesRequest};
use primitives::PriceFull;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use storage::models::Price;

use crate::response::ResponseResults;

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_asset_price(
    asset_id: String,
    currency: Option<String>,
    price_client: &State<Mutex<PriceClient>>,
) -> Json<AssetPrices> {
    let request = Json(AssetPricesRequest {
        asset_ids: vec![asset_id],
        currency,
    });
    get_assets_prices(request, price_client).await
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(
    request: Json<AssetPricesRequest>,
    price_client: &State<Mutex<PriceClient>>,
) -> Json<AssetPrices> {
    let currency: String = request
        .currency
        .clone()
        .unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());
    let asset_ids = request.asset_ids.iter().map(|x| x.as_str()).collect();
    let prices_result = price_client
        .lock()
        .await
        .get_cache_prices(currency.clone().as_str(), asset_ids)
        .await;
    let prices = price_response(prices_result.unwrap_or_default());
    Json(AssetPrices {
        currency: currency.clone(),
        prices,
    })
}

#[get("/prices/list")]
pub async fn get_prices_list(
    price_client: &State<Mutex<PriceClient>>,
) -> Json<ResponseResults<PriceFull>> {
    let results = price_client
        .lock()
        .await
        .get_prices_list()
        .await
        .unwrap_or_default();
    Json(ResponseResults { results })
}

fn price_response(prices: Vec<Price>) -> Vec<AssetPrice> {
    let mut response = Vec::new();
    for asset_price in prices {
        response.push(asset_price.as_primitive());
    }
    response
}
