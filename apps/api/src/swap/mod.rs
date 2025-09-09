pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::SwapClient;
use primitives::fiat_assets::FiatAssets;
use rocket::{get, tokio::sync::Mutex, State};

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<crate::SwapClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.lock().await.get_swap_assets().await?.into())
}
