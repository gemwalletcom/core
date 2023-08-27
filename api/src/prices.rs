extern crate rocket;
use pricer::DEFAULT_FIAT_CURRENCY;
use rocket::serde::json::Json;
use pricer::client::Client as PriceClient;
use storage::models::Price;
use rocket::State;
use rocket::tokio::sync::Mutex;
use primitives::asset_price::{AssetPrices, AssetPrice, AssetPricesRequest};

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_asset_price(asset_id: String, currency: Option<String>, price_client: &State<Mutex<PriceClient>>) -> Json<AssetPrices> {
    let request = Json(AssetPricesRequest{asset_ids: vec![asset_id], currency});
    get_assets_prices(request, price_client).await
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(request: Json<AssetPricesRequest>, price_client: &State<Mutex<PriceClient>>) -> Json<AssetPrices> {
    let currency: String = request.currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());    
    let asset_ids = request.asset_ids.iter().map(|x| x.as_str()).collect();
    let prices_result = price_client.lock().await.get_cache_prices(currency.clone().as_str(), asset_ids).await;
    let prices = price_response(prices_result.unwrap_or_default());
    Json(AssetPrices{currency: currency.clone(), prices})
}

fn price_response(prices: Vec<Price>) -> Vec<AssetPrice> {
    let mut response = Vec::new();
    for asset_price in prices {
        let price_response = AssetPrice{
            asset_id: asset_price.asset_id,
            price: asset_price.price,
            price_change_percentage_24h: asset_price.price_change_percentage_24h,
        };
        response.push(price_response);
    }
    response
}