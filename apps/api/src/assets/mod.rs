pub mod cilent;
use crate::responders::{ApiError, ApiResponse};
pub use cilent::{AssetsClient, AssetsSearchClient};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

use std::str::FromStr;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, Chain};

#[get("/assets/<asset_id>")]
pub async fn get_asset(asset_id: &str, client: &State<Mutex<AssetsClient>>) -> Result<ApiResponse<AssetFull>, ApiError> {
    Ok(client.lock().await.get_asset_full(asset_id)?.into())
}

#[post("/assets", format = "json", data = "<asset_ids>")]
pub async fn get_assets(asset_ids: Json<Vec<String>>, client: &State<Mutex<AssetsClient>>) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    Ok(client.lock().await.get_assets(asset_ids.0)?.into())
}

#[post("/assets/add", format = "json", data = "<asset_id>")]
pub async fn add_asset(
    asset_id: Json<Vec<AssetId>>,
    client: &State<Mutex<AssetsClient>>,
    chain_client: &State<Mutex<crate::chain::ChainClient>>,
) -> Result<ApiResponse<Vec<Asset>>, ApiError> {
    let asset_id = asset_id.0.first().ok_or(ApiError::BadRequest("Missing asset_id".to_string()))?;

    let asset = chain_client
        .lock()
        .await
        .get_token_data(
            asset_id.chain,
            asset_id.token_id.clone().ok_or(ApiError::BadRequest("Missing token_id".to_string()))?,
        )
        .await?;
    client.lock().await.add_assets(vec![asset.clone()])?;

    Ok(vec![asset].into())
}

#[get("/assets/search?<query>&<chains>&<tags>&<limit>&<offset>")]
pub async fn get_assets_search(
    query: String,
    chains: Option<String>,
    tags: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    client: &State<Mutex<AssetsSearchClient>>,
) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let chains = chains
        .unwrap_or_default()
        .split(',')
        .flat_map(Chain::from_str)
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let tags = tags
        .unwrap_or_default()
        .split(',')
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    Ok(client
        .lock()
        .await
        .get_assets_search(query.as_str(), chains.clone(), tags.clone(), limit.unwrap_or(50), offset.unwrap_or(0))
        .await?
        .into())
}

#[get("/assets/device/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Result<ApiResponse<Vec<AssetId>>, ApiError> {
    Ok(client.lock().await.get_assets_by_device_id(device_id, wallet_index, from_timestamp)?.into())
}
