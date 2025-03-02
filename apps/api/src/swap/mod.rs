pub mod client;

extern crate rocket;
use primitives::fiat_assets::FiatAssets;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

pub use client::SwapClient;

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<crate::SwapClient>>) -> Json<FiatAssets> {
    let quote = client.lock().await.get_swap_assets().await.unwrap();
    Json(quote)
}
