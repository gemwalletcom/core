extern crate rocket;
use primitives::fiat_assets::FiatAssets;
use primitives::{SwapQuoteResult, SwapQuoteRequest};
use rocket::serde::json::Json;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/swap/quote?<quote..>")]
pub async fn get_swap_quote(
    quote: SwapQuoteRequest, 
    client: &State<Mutex<crate::SwapClient>>,
) -> Json<SwapQuoteResult> {
    let quote = client.lock().await.swap_quote(quote).await.unwrap();
    Json(quote)
}

#[get("/swap/assets")]
pub async fn get_swap_assets(
    client: &State<Mutex<crate::SwapClient>>,
) -> Json<FiatAssets> {
    let quote = client.lock().await.get_swap_assets().await.unwrap();
    Json(quote)
}