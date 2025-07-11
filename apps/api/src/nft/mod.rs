extern crate rocket;
use std::str::FromStr;

use primitives::{Chain, NFTAsset, NFTData, ResponseResult};
use rocket::{response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};
use nft_client::NFTClient;

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

use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::Request;
use std::collections::HashMap;
use std::io::Cursor;

// from db

#[get("/nft/collections/<collection_id>")]
pub async fn get_nft_collection(collection_id: &str, client: &State<Mutex<NFTClient>>) -> Result<Json<ResponseResult<NFTData>>, NotFound<String>> {
    let result = client.lock().await.get_nft_collection_data(collection_id);
    match result {
        Ok(data) => Ok(Json(ResponseResult::new(data))),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

#[get("/nft/assets/<asset_id>/image_preview")]
pub async fn get_nft_asset_image_preview(asset_id: &str, client: &State<Mutex<NFTClient>>) -> Result<ImageResponse, NotFound<String>> {
    let (image_data, content_type, upstream_headers) = client.lock().await.get_nft_asset_image(asset_id).await.map_err(|e| NotFound(e.to_string()))?;
    let content_type = ContentType::parse_flexible(content_type.as_ref().unwrap_or(&"image/png".to_string())).unwrap_or(ContentType::PNG);
    let cache_control = upstream_headers
        .get("cache-control")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "public, max-age=604800, immutable".to_string());
    let last_modified = upstream_headers.get("last-modified").map(|s| s.to_string());
    let mut headers = HashMap::new();
    headers.insert("cache-control".to_string(), cache_control);
    if let Some(last_modified) = last_modified {
        headers.insert("last-modified".to_string(), last_modified);
    }
    Ok(ImageResponse::new(image_data, content_type, headers))
}

pub struct ImageResponse {
    data: Vec<u8>,
    content_type: ContentType,
    headers: HashMap<String, String>,
}

impl ImageResponse {
    pub fn new(data: Vec<u8>, content_type: ContentType, headers: HashMap<String, String>) -> Self {
        Self {
            data,
            content_type,
            headers: headers.into_iter().map(|(k, v)| (k.to_lowercase(), v)).collect(),
        }
    }
}

impl<'r> Responder<'r, 'static> for ImageResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let mut response = response::Response::build();
        response.header(self.content_type);
        for (name, value) in self.headers {
            response.raw_header(name, value);
        }
        response.sized_body(self.data.len(), Cursor::new(self.data));
        response.ok()
    }
}
