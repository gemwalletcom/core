extern crate rocket;
use fiat::client::Client as FiatProvider;
use primitives::{
    fiat_assets::FiatAssets, fiat_quote::FiatQuotes, fiat_quote_request::FiatBuyRequest,
};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

// on ramp

#[get("/fiat/on_ramp/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>")]
pub async fn get_fiat_on_ramp_quotes(
    asset_id: String,
    amount: f64,
    currency: String,
    wallet_address: String,
    ip_address: Option<String>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatProvider>>,
) -> Json<FiatQuotes> {
    get_fiat_quotes(
        asset_id,
        amount,
        currency,
        wallet_address,
        ip_address,
        ip,
        fiat_client,
    )
    .await
}

#[get("/fiat/on_ramp/assets")]
pub async fn get_fiat_on_ramp_assets(fiat_client: &State<Mutex<FiatProvider>>) -> Json<FiatAssets> {
    get_fiat_assets(fiat_client).await
}

// off ramp

// old routes. switched to use on_ramp and off_ramp routes

#[get("/fiat/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>")]
pub async fn get_fiat_quotes(
    asset_id: String,
    amount: f64,
    currency: String,
    wallet_address: String,
    ip_address: Option<String>,
    ip: std::net::IpAddr,
    fiat_client: &State<Mutex<FiatProvider>>,
) -> Json<FiatQuotes> {
    let request: FiatBuyRequest = FiatBuyRequest {
        asset_id: asset_id.clone(),
        ip_address: ip_address.unwrap_or(ip.to_string()),
        fiat_amount: amount,
        fiat_currency: currency,
        wallet_address,
    };
    let quotes = fiat_client.lock().await.get_quotes(request).await;
    match quotes {
        Ok(value) => Json(FiatQuotes { quotes: value }),
        Err(_) => Json(FiatQuotes { quotes: vec![] }),
    }
}

#[get("/fiat/assets")]
pub async fn get_fiat_assets(fiat_client: &State<Mutex<FiatProvider>>) -> Json<FiatAssets> {
    let assets = fiat_client.lock().await.get_assets().await.unwrap();
    Json(assets)
}

#[post("/fiat/webhooks/<provider>", format = "json", data = "<data>")]
pub async fn create_fiat_webhook(
    provider: &str,
    data: Json<serde_json::Value>,
    fiat_client: &State<Mutex<FiatProvider>>,
) -> Json<bool> {
    print!(
        "webhook: {}, data: {:?}",
        provider,
        serde_json::to_string_pretty(&data.0)
    );
    let result = fiat_client
        .lock()
        .await
        .create_fiat_webhook(provider, data.into_inner())
        .await
        .unwrap();
    Json(result)
}
