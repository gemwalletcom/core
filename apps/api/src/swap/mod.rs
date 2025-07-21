pub mod client;

use primitives::fiat_assets::FiatAssets;
use rocket::{get, serde::json::Json, tokio::sync::Mutex, State};

pub use client::SwapClient;

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<crate::SwapClient>>) -> Json<FiatAssets> {
    let quote = client.lock().await.get_swap_assets().await.unwrap();
    Json(quote)
}
