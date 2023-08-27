extern crate rocket;
use rocket::State;
use rocket::serde::json::Json;
use primitives::{fiat_quote::FiatQuotes, fiat_quote_request::FiatBuyRequest, fiat_assets::FiatAssets};
use fiat::model::FiatRates;
use fiat::client::Client as FiatClient;
use rocket::tokio::sync::Mutex;

#[get("/fiat/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>")]
pub async fn get_fiat_quotes(
    asset_id: String,
    amount: f64, 
    currency: String, 
    wallet_address: String,
    ip_address: Option<String>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatQuotes> {
    let request: FiatBuyRequest = FiatBuyRequest{
        asset_id: asset_id.clone(), 
        ip_address: ip_address.unwrap_or(ip.to_string()),
        fiat_amount: amount,
        fiat_currency: currency,
        wallet_address,
    };
    let fiat_mapping = fiat_client.lock().await.get_fiat_mapping(asset_id.clone().as_str()).unwrap_or_default();
    let quotes = fiat_client.lock().await.get_quotes(request, fiat_mapping).await;
    match quotes {
       Ok(value) => Json(FiatQuotes{quotes: value}),
       Err(_) => Json(FiatQuotes{quotes: vec![]}),
    }
}

#[get("/fiat/assets")]
pub async fn get_fiat_assets(
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatAssets> {
    let assets = fiat_client.lock().await.get_assets().await;
    match assets {
        Ok(value) => Json(value),
        Err(_) => Json(FiatAssets{version: 0, asset_ids: vec![]}),
    }
}

#[get("/fiat/rates")]
pub async fn get_fiat_rates(
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatRates> {
    let rates = fiat_client.lock().await.get_fiat_rates().await;
    match rates {
        Ok(value) => Json(value),
        Err(_) => Json(FiatRates{rates: vec![]}),
    }
}