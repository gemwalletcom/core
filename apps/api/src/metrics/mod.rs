mod client;
mod fiat;
mod parser;
mod price;

use crate::responders::ApiError;
pub use client::MetricsClient;
pub use fiat::{metrics_fiat_quote_url, metrics_fiat_quotes};
use rocket::{State, get, response::content::RawText, tokio::sync::Mutex};

#[get("/")]
pub async fn get_metrics(client: &State<Mutex<MetricsClient>>) -> Result<RawText<String>, ApiError> {
    Ok(RawText(client.lock().await.get()))
}
