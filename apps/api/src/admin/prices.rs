use rocket::{State, post, serde::json::Json};
use streamer::{FetchPricesPayload, StreamProducer, StreamProducerQueue};

use crate::admin::AdminAuthorized;
use crate::responders::{ApiError, ApiResponse};

#[post("/prices/add", format = "json", data = "<payload>")]
pub async fn add_price(_admin: AdminAuthorized, payload: Json<FetchPricesPayload>, stream_producer: &State<StreamProducer>) -> Result<ApiResponse<FetchPricesPayload>, ApiError> {
    let payload = payload.into_inner();
    stream_producer.publish_fetch_prices(payload.clone()).await?;
    Ok(payload.into())
}
