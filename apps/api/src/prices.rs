extern crate rocket;
use pricer::client::PriceClient;
use primitives::asset_price::{AssetPrices, AssetPricesRequest};
use primitives::DEFAULT_FIAT_CURRENCY;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

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
    let prices = price_client
        .lock()
        .await
        .get_cache_prices(currency.clone().as_str(), asset_ids)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|x| x.as_primitive())
        .collect();

    Json(AssetPrices {
        currency: currency.clone(),
        prices,
    })
}
