use crate::responders::{ApiError, ApiResponse};
use pricer::MarketsClient;
use primitives::Markets;
use rocket::{State, get, tokio::sync::Mutex};

#[get("/markets")]
pub async fn get_markets(client: &State<Mutex<MarketsClient>>) -> Result<ApiResponse<Markets>, ApiError> {
    Ok(client.lock().await.get_markets().await?.into())
}
