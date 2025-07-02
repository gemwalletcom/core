extern crate rocket;
use primitives::{Chain, NFTAsset, NFTData, ResponseResult};
use rocket::response::Redirect;
use rocket::{response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};
use std::str::FromStr;
pub mod client;
pub use self::client::NFTClient;

// by device

#[get("/nft/assets/device/<device_id>?<wallet_index>")]
pub async fn get_nft_assets(
    device_id: &str,
    wallet_index: i32,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResult<Vec<NFTData>>>, NotFound<String>> {
    let result = client.lock().await.get_nft_assets(device_id, wallet_index).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

// by address. mostly for testing purposes

#[get("/nft/assets/chain/<chain>?<address>")]
pub async fn get_nft_assets_by_chain(
    chain: &str,
    address: &str,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResult<Vec<NFTData>>>, NotFound<String>> {
    let chain = Chain::from_str(chain).unwrap();
    let result = client.lock().await.get_nft_assets_by_chain(chain, address).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

// collections

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(collection_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Json<ResponseResult<bool>>, NotFound<String>> {
    let result = client.lock().await.update_collection(collection_id).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

// assets

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(asset_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Json<ResponseResult<bool>>, NotFound<String>> {
    let result = client.lock().await.update_asset(asset_id).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

#[get("/nft/assets/<asset_id>")]
pub async fn get_nft_asset(asset_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Json<ResponseResult<NFTAsset>>, NotFound<String>> {
    let result = client.lock().await.get_nft_asset(asset_id);
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

#[get("/nft/assets/<asset_id>/image_preview")]
pub async fn get_nft_asset_image_preview(asset_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Redirect, NotFound<String>> {
    let result = client.lock().await.get_nft_asset(asset_id).unwrap();
    Ok(Redirect::to(result.images.preview.url))
}

// from db

#[get("/nft/collections/<collection_id>")]
pub async fn get_nft_collection(collection_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Json<ResponseResult<NFTData>>, NotFound<String>> {
    let result = client.lock().await.get_nft_collection_data(collection_id);
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
