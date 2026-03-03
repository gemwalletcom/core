pub mod client;
pub mod near_intents;
use crate::responders::{ApiError, ApiResponse};
pub use client::SwapClient;
pub use near_intents::NearIntentsProxyClient;

use primitives::fiat_assets::FiatAssets;
use rocket::{State, get, tokio::sync::Mutex};

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<crate::SwapClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.lock().await.get_swap_assets().await?.into())
}
