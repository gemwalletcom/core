mod client;

use crate::params::FiatQuoteTypeParam;
use crate::responders::{ApiError, ApiResponse};
pub use client::FiatQuotesClient;
pub use fiat::{FiatProviderFactory, IPAddressInfo, IPCheckClient};
use primitives::{FiatAssets, FiatQuoteType, FiatQuoteUrl, FiatQuotes, FiatQuotesData};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};
use std::str::FromStr;
use streamer::FiatWebhook;

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
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let quote_type = FiatQuoteType::from_str(r#type).unwrap_or(FiatQuoteType::Buy);
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    Ok(client
        .lock()
        .await
        .get_quotes(asset_id, fiat_amount, crypto_value, quote_type, currency, wallet_address, &ip_addr, provider_id)
        .await?
        .into())
}

#[get("/fiat/quotes/<quote_type>/<asset_id>?<fiat_amount>&<currency>&<ip_address>&<provider_id>")]
pub async fn get_fiat_quotes_by_type(
    quote_type: FiatQuoteTypeParam,
    asset_id: &str,
    fiat_amount: f64,
    currency: &str,
    ip_address: Option<&str>,
    provider_id: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotesData>, ApiError> {
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    Ok(client
        .lock()
        .await
        .get_quotes_data(asset_id, fiat_amount, quote_type.0, currency, &ip_addr, provider_id)
        .await?
        .into())
}

#[post("/fiat/quotes/url?<quote_id>&<wallet_address>&<device_id>")]
pub async fn get_fiat_quote_url(
    quote_id: &str,
    wallet_address: &str,
    device_id: &str,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuoteUrl>, ApiError> {
    Ok(client.lock().await.get_quote_url(quote_id, wallet_address, &ip.to_string(), device_id).await?.into())
}

#[get("/fiat/on_ramp/quotes/<asset_id>?<amount>&<currency>&<wallet_address>&<ip_address>&<provider_id>")]
pub async fn get_fiat_on_ramp_quotes(
    asset_id: &str,
    amount: f64,
    currency: &str,
    wallet_address: &str,
    ip_address: Option<&str>,
    provider_id: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let ip_addr = ip_address.unwrap_or(&ip.to_string()).to_string();
    Ok(client
        .lock()
        .await
        .get_quotes(
            asset_id,
            Some(amount),
            None,
            FiatQuoteType::Buy,
            currency,
            wallet_address,
            &ip_addr,
            provider_id,
        )
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
