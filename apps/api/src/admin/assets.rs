use primitives::AssetId;
use rocket::{State, post, serde::json::Json};
use streamer::{StreamProducer, StreamProducerQueue};

use crate::admin::AdminAuthorized;
use crate::responders::{ApiError, ApiResponse};

#[post("/assets/add", format = "json", data = "<asset_id>")]
pub async fn add_asset(_admin: AdminAuthorized, asset_id: Json<AssetId>, stream_producer: &State<StreamProducer>) -> Result<ApiResponse<AssetId>, ApiError> {
    let asset_id = asset_id.into_inner();
    stream_producer.publish_fetch_assets(vec![asset_id.clone()]).await?;
    Ok(asset_id.into())
}
