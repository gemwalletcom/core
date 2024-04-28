extern crate rocket;
use crate::response::ResponseError;
use primitives::fiat_assets::FiatAssets;
use primitives::{SwapQuoteRequest, SwapQuoteResult};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/swap/quote?<quote..>")]
pub async fn get_swap_quote(
    quote: SwapQuoteRequest,
    client: &State<Mutex<crate::SwapClient>>,
) -> Result<Json<SwapQuoteResult>, Json<ResponseError>> {
    client
        .lock()
        .await
        .swap_quote(quote)
        .await
        .map(Json)
        .map_err(|err| Json(err.into()))
}

#[post("/swap/quote", format = "json", data = "<quote>")]
pub async fn post_swap_quote(
    quote: Json<SwapQuoteRequest>,
    client: &State<Mutex<crate::SwapClient>>,
) -> Result<Json<SwapQuoteResult>, Json<ResponseError>> {
    client
        .lock()
        .await
        .swap_quote(quote.0)
        .await
        .map(Json)
        .map_err(|err| Json(err.into()))
}

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<Mutex<crate::SwapClient>>) -> Json<FiatAssets> {
    let quote = client.lock().await.get_swap_assets().await.unwrap();
    Json(quote)
}
