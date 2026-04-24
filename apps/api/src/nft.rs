use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};
use ::nft::NFTClient;
use rocket::Request;
use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::{State, get, put, tokio::sync::Mutex};
use std::collections::HashMap;
use std::io::Cursor;

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(collection_id: NftCollectionIdParam, client: &State<Mutex<NFTClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.update_collection(&collection_id.0.id()).await?.into())
}

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(asset_id: NftAssetIdParam, client: &State<Mutex<NFTClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.update_asset(asset_id.0.as_ref()).await?.into())
}

#[get("/nft/assets/<asset_id>/image_preview")]
pub async fn get_nft_asset_image_preview(asset_id: NftAssetIdParam, client: &State<Mutex<NFTClient>>) -> Result<ImageResponse, ApiError> {
    let (image_data, content_type, upstream_headers) = client.lock().await.get_nft_asset_image(asset_id.0.as_ref()).await?;
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
