mod client;

use crate::metrics::{metrics_fiat_quote_url, metrics_fiat_quotes};
use crate::params::{CurrencyParam, FiatQuoteTypeParam};
use crate::responders::{ApiError, ApiResponse};
pub use client::FiatQuotesClient;
pub use fiat::{FiatProviderFactory, IPAddressInfo, IPCheckClient};
use primitives::{FiatAssets, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlRequest, FiatQuotes, FiatQuotesOld};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};
use std::str::FromStr;
use streamer::FiatWebhook;

const DEBUG_FIAT_IP: &str = "210.138.184.59";

#[get("/fiat/quotes/<asset_id>?<fiat_amount>&<crypto_value>&<type>&<currency>&<wallet_address>&<ip_address>&<provider>")]
pub async fn get_fiat_quotes(
    asset_id: &str,
    fiat_amount: Option<f64>,
    crypto_value: Option<&str>,
    r#type: &str,
    currency: CurrencyParam,
    wallet_address: &str,
    ip_address: Option<&str>,
    provider: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotesOld>, ApiError> {
    let quote_type = FiatQuoteType::from_str(r#type).unwrap_or(FiatQuoteType::Buy);
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    Ok(client
        .lock()
        .await
        .get_quotes_old(
            asset_id,
            fiat_amount,
            crypto_value,
            quote_type,
            &currency.as_string(),
            wallet_address,
            &ip_addr,
            provider,
        )
        .await?
        .into())
}

#[get("/fiat/quotes/<quote_type>/<asset_id>?<amount>&<currency>&<ip_address>&<provider>")]
pub async fn get_fiat_quotes_by_type(
    quote_type: FiatQuoteTypeParam,
    asset_id: &str,
    amount: f64,
    currency: CurrencyParam,
    ip_address: Option<&str>,
    provider: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    let request = FiatQuoteRequest {
        asset_id: asset_id.to_string(),
        quote_type: quote_type.0,
        amount,
        currency: currency.as_string(),
        provider_id: provider.map(|x| x.to_string()),
        ip_address: ip_addr,
    };
    let quotes = client.lock().await.get_quotes(request).await?;
    metrics_fiat_quotes(&quotes);
    Ok(quotes.into())
}

#[post("/fiat/quotes/url", data = "<request>")]
pub async fn get_fiat_quote_url(
    request: Json<FiatQuoteUrlRequest>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuoteUrl>, ApiError> {
    let ip_address = if cfg!(debug_assertions) { DEBUG_FIAT_IP } else { &ip.to_string() };
    let (url, quote) = client.lock().await.get_quote_url(&request, ip_address).await?;
    metrics_fiat_quote_url(&quote);
    Ok(url.into())
}

#[get("/fiat/on_ramp/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>&<provider>")]
pub async fn get_fiat_on_ramp_quotes(
    asset_id: &str,
    amount: f64,
    currency: &str,
    wallet_address: &str,
    ip_address: Option<&str>,
    provider: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotesOld>, ApiError> {
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    Ok(client
        .lock()
        .await
        .get_quotes_old(asset_id, Some(amount), None, FiatQuoteType::Buy, currency, wallet_address, &ip_addr, provider)
        .await?
        .into())
}

#[get("/fiat/on_ramp/assets")]
pub async fn get_fiat_on_ramp_assets(client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.lock().await.get_on_ramp_assets().await?.into())
}

#[get("/fiat/off_ramp/assets")]
pub async fn get_fiat_off_ramp_assets(client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.lock().await.get_off_ramp_assets().await?.into())
}

#[post("/fiat/webhooks/<provider>", data = "<webhook_data>")]
pub async fn create_fiat_webhook(
    provider: &str,
    webhook_data: Json<serde_json::Value>,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatWebhook>, ApiError> {
    Ok(client.lock().await.process_and_publish_webhook(provider, webhook_data.0).await?.payload.into())
}

#[get("/fiat/orders/<provider>/<order_id>")]
pub async fn get_fiat_order(
    provider: &str,
    order_id: &str,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<primitives::FiatTransaction>, ApiError> {
    Ok(client.lock().await.get_order_status(provider, order_id).await?.into())
}

#[get("/ip")]
pub async fn get_ip_address(ip: std::net::IpAddr, ip_check_client: &State<Mutex<IPCheckClient>>) -> Result<ApiResponse<IPAddressInfo>, ApiError> {
    Ok(ip_check_client.lock().await.get_ip_address(&ip.to_string()).await?.into())
}
