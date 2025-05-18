extern crate rocket;
use std::str::FromStr;

pub use fiat::{FiatClient, FiatProviderFactory};

use primitives::{FiatAssets, FiatQuoteRequest, FiatQuoteType, FiatQuotes};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

// on ramp

#[get("/fiat/quotes/<asset_id>?<fiat_amount>&<crypto_value>&<type>&<currency>&<wallet_address>&<ip_address>&<provider_id>")]
pub async fn get_fiat_quotes(
    asset_id: &str,
    fiat_amount: Option<f64>,
    crypto_value: Option<&str>,
    r#type: &str,
    currency: &str,
    wallet_address: &str,
    ip_address: Option<&str>,
    provider_id: Option<&str>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatClient>>,
) -> Json<FiatQuotes> {
    let quote_type = FiatQuoteType::from_str(r#type).unwrap_or(FiatQuoteType::Buy);
    if fiat_amount.is_none() && crypto_value.is_none() {
        return Json(FiatQuotes {
            quotes: vec![],
            errors: vec![],
        });
    }
    let request: FiatQuoteRequest = FiatQuoteRequest {
        asset_id: asset_id.to_string(),
        quote_type,
        ip_address: ip_address.unwrap_or(&ip.to_string()).to_string(),
        fiat_amount,
        fiat_currency: currency.to_string(),
        crypto_value: crypto_value.map(|x| x.to_string()),
        wallet_address: wallet_address.to_string(),
        provider_id: provider_id.map(|x| x.to_string()),
    };
    match fiat_client.lock().await.get_quotes(request).await {
        Ok(value) => Json(value),
        Err(_) => Json(FiatQuotes {
            quotes: vec![],
            errors: vec![],
        }),
    }
}

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
        asset_id,
        quote_type: FiatQuoteType::Buy,
        ip_address: ip_address.unwrap_or(ip.to_string()),
        fiat_amount: Some(amount),
        fiat_currency: currency,
        crypto_value: None,
        wallet_address,
        provider_id,
    };
    let result = fiat_client.lock().await.get_quotes(request).await;
    match result {
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
    println!("webhook: {}, data: {}", provider, serde_json::to_string(&data.0).unwrap());
    let result = fiat_client.lock().await.create_fiat_webhook(provider, data.into_inner()).await.unwrap();
    Json(result)
}
