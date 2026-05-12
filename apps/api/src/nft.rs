use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::ApiError;
use ::nft::NFTClient;
use primitives::NFTResource;
use rocket::serde::json::Json;
use rocket::{State, get};

#[get("/nft/assets/<asset_id>/preview")]
pub async fn get_nft_asset_preview(asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    let identifier = asset_id.0.to_string();
    Ok(Json(client.load_nft_asset(&identifier)?.images.preview))
}

#[get("/nft/assets/<asset_id>/resource")]
pub async fn get_nft_asset_resource(asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    let identifier = asset_id.0.to_string();
    Ok(Json(client.load_nft_asset(&identifier)?.resource))
}

#[get("/nft/collections/<collection_id>/preview")]
pub async fn get_nft_collection_preview(collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_collection(&collection_id.0.to_string())?.images.preview))
}
