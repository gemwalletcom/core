use crate::devices::FiatQuotesClient;
use crate::responders::{ApiError, ApiResponse};
use rocket::{State, get, tokio::sync::Mutex};

#[get("/fiat/orders/<provider>/<order_id>")]
pub async fn get_fiat_order_v1(provider: &str, order_id: &str, client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<primitives::FiatTransaction>, ApiError> {
    Ok(client.lock().await.get_order_status(provider, order_id).await?.into())
}
