extern crate rocket;
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