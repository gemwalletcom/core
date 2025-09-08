use crate::responders::ApiError;
use gem_tracing::error_with_context;
use pricer::price_client::PriceClient;
use pricer::ChartClient;
use primitives::{AssetIdVecExt, AssetMarketPrice, AssetPrices, AssetPricesRequest, ChartPeriod, Charts, FiatRate, DEFAULT_FIAT_CURRENCY};
use rocket::{get, http::Status, post, serde::json::Json, tokio::sync::Mutex, State};

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_price(asset_id: &str, currency: Option<&str>, price_client: &State<Mutex<PriceClient>>) -> Result<Json<AssetMarketPrice>, ApiError> {
    let currency = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);

    match price_client.lock().await.get_asset_price(asset_id, currency).await {
        Ok(price) => Ok(Json(price)),
        Err(error) => {
            error_with_context("get_asset_price failed", error.as_ref(), &[("asset_id", asset_id)]);
            Err(error.into())
        }
    }
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(request: Json<AssetPricesRequest>, price_client: &State<Mutex<PriceClient>>) -> Result<Json<AssetPrices>, Status> {
    let currency: String = request.currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());

    let asset_ids = request.asset_ids.ids();
    let asset_count = asset_ids.len();
    match price_client.lock().await.get_asset_prices(currency.as_str(), asset_ids).await {
        Ok(prices) => Ok(Json(prices)),
        Err(error) => {
            error_with_context("get_asset_prices failed", error.as_ref(), &[("count", &asset_count.to_string())]);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/fiat_rates")]
pub async fn get_fiat_rates(price_client: &State<Mutex<PriceClient>>) -> Result<Json<Vec<FiatRate>>, Status> {
    match price_client.lock().await.get_fiat_rates() {
        Ok(rates) => Ok(Json(rates)),
        Err(error) => {
            error_with_context("get_fiat_rates failed", error.as_ref(), &[]);
            Err(Status::InternalServerError)
        }
    }
}

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
