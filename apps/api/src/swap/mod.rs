pub mod client;
pub mod near_intents;
pub mod okx;

pub use client::SwapClient;
pub use near_intents::NearIntentsProxyClient;

use crate::responders::{ApiError, ApiResponse};
use primitives::FiatAssets;
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<SwapClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.lock().await.get_swap_assets().await?.into())
}

#[post("/swaps/near_intents/quote", data = "<body>")]
pub async fn post_near_intents_quote(body: Json<serde_json::Value>, client: &State<Mutex<NearIntentsProxyClient>>) -> Result<Json<serde_json::Value>, ApiError> {
    let response = client.lock().await.quote(body.0).await?;
    Ok(Json(response))
}
