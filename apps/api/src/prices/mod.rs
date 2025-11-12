use crate::responders::{ApiError, ApiResponse};
use pricer::ChartClient;
use pricer::price_client::PriceClient;
use primitives::{AssetIdVecExt, AssetMarketPrice, AssetPrices, AssetPricesRequest, ChartPeriod, Charts, DEFAULT_FIAT_CURRENCY, FiatRate};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_price(asset_id: &str, currency: Option<&str>, price_client: &State<Mutex<PriceClient>>) -> Result<ApiResponse<AssetMarketPrice>, ApiError> {
    let currency = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);
    Ok(price_client.lock().await.get_asset_price(asset_id, currency).await?.into())
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(request: Json<AssetPricesRequest>, price_client: &State<Mutex<PriceClient>>) -> Result<ApiResponse<AssetPrices>, ApiError> {
    let currency: String = request.currency.clone().unwrap_or(DEFAULT_FIAT_CURRENCY.to_string());
    let asset_ids = request.asset_ids.ids();
    Ok(price_client.lock().await.get_asset_prices(currency.as_str(), asset_ids).await?.into())
}

#[get("/fiat_rates")]
pub async fn get_fiat_rates(price_client: &State<Mutex<PriceClient>>) -> Result<ApiResponse<Vec<FiatRate>>, ApiError> {
    Ok(price_client.lock().await.get_fiat_rates()?.into())
}

#[get("/charts/<asset_id>?<period>&<currency>")]
pub async fn get_charts(
    asset_id: &str,
    period: Option<&str>,
    currency: Option<&str>,
    charts_client: &State<Mutex<ChartClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Result<ApiResponse<Charts>, ApiError> {
    let period = ChartPeriod::new(period.unwrap_or_default().to_string()).unwrap_or(ChartPeriod::Day);
    let currency_value = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);

    let coin_id = charts_client.lock().await.get_coin_id(asset_id)?;

    let prices = charts_client.lock().await.get_charts_prices(&coin_id, period, currency_value).await?;

    let asset_price = price_client.lock().await.get_asset_price(asset_id, currency_value).await?;

    let response = Charts {
        price: asset_price.price,
        market: asset_price.market,
        prices,
        market_caps: vec![],
        total_volumes: vec![],
    };

    Ok(response.into())
}
