use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};
use ::nft::NFTClient;
use primitives::NFTResource;
use rocket::serde::json::Json;
use rocket::{State, get, put};

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_collection(&collection_id.0.id()).await?.into())
}

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_asset(asset_id.0.as_ref()).await?.into())
}

#[get("/nft/assets/<asset_id>/preview")]
pub async fn get_nft_asset_preview(asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_asset(asset_id.0.as_ref())?.images.preview))
}

#[get("/nft/assets/<asset_id>/resource")]
pub async fn get_nft_asset_resource(asset_id: NftAssetIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_asset(asset_id.0.as_ref())?.resource))
}

#[get("/nft/collections/<collection_id>/preview")]
pub async fn get_nft_collection_preview(collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_collection(&collection_id.0.id())?.images.preview))
}
