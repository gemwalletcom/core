extern crate rocket;
pub use fiat::{FiatClient, FiatProviderFactory};

use primitives::{FiatAssets, FiatQuoteRequest, FiatQuotes};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
// on ramp

#[get("/fiat/on_ramp/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>&<provider_id>")]
pub async fn get_fiat_on_ramp_quotes(
    asset_id: String,
    amount: f64,
    currency: String,
    wallet_address: String,
    ip_address: Option<String>,
    provider_id: Option<String>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatQuotes> {
    let request: FiatQuoteRequest = FiatQuoteRequest {
        asset_id: asset_id.clone(),
        ip_address: ip_address.unwrap_or(ip.to_string()),
        fiat_amount: amount,
        fiat_currency: currency,
        crypto_amount: None,
        wallet_address,
        provider_id,
    };
    let result = fiat_client.lock().await.get_buy_quotes(request).await;
    match result {
        Ok(value) => Json(value),
        Err(_) => Json(FiatQuotes {
            quotes: vec![],
            errors: vec![],
        }),
    }
}

#[get("/fiat/off_ramp/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>&<provider_id>")]
pub async fn get_fiat_off_ramp_quotes(
    asset_id: &str,
    amount: f64,
    currency: &str,
    wallet_address: &str,
    ip_address: Option<String>,
    provider_id: Option<String>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatQuotes> {
    let request = FiatQuoteRequest {
        asset_id: asset_id.into(),
        ip_address: ip_address.unwrap_or(ip.to_string()),
        fiat_amount: 0.0,
        crypto_amount: Some(amount),
        fiat_currency: currency.into(),
        wallet_address: wallet_address.into(),
        provider_id,
    };
    let quotes = fiat_client.lock().await.get_sell_quotes(request).await;
    match quotes {
        Ok(value) => Json(value),
        Err(_) => Json(FiatQuotes {
            quotes: vec![],
            errors: vec![],
        }),
    }
}

#[get("/fiat/on_ramp/assets")]
pub async fn get_fiat_on_ramp_assets(fiat_client: &State<Mutex<FiatClient>>) -> Json<FiatAssets> {
    let assets = fiat_client.lock().await.get_on_ramp_assets().await.unwrap();
    Json(assets)
}

#[get("/fiat/off_ramp/assets")]
pub async fn get_fiat_off_ramp_assets(fiat_client: &State<Mutex<FiatClient>>) -> Json<FiatAssets> {
    let assets = fiat_client.lock().await.get_off_ramp_assets().await.unwrap();
    Json(assets)
}

#[post("/fiat/webhooks/<provider>", format = "json", data = "<data>")]
pub async fn create_fiat_webhook(provider: &str, data: Json<serde_json::Value>, fiat_client: &State<Mutex<FiatClient>>) -> Json<bool> {
    print!("webhook: {}, data: {:?}", provider, serde_json::to_string_pretty(&data.0));
    let result = fiat_client.lock().await.create_fiat_webhook(provider, data.into_inner()).await.unwrap();
    Json(result)
}
