use ::nft::NFTClient;
use rocket::{State, put};
use streamer::{StreamProducer, StreamProducerQueue};

use crate::admin::AdminAuthorized;
use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(_admin: AdminAuthorized, collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_collection(&collection_id.0.to_string()).await?.into())
}

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(_admin: AdminAuthorized, asset_id: NftAssetIdParam, stream_producer: &State<StreamProducer>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(stream_producer.publish_fetch_nft_asset(asset_id.0).await?.into())
}
