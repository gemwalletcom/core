extern crate rocket;

use std::str::FromStr;

use crate::asset_client::{AssetsChainProvider, AssetsSearchClient};
use crate::AssetsClient;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, Chain};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/assets/<asset_id>")]
pub async fn get_asset(asset_id: &str, client: &State<Mutex<AssetsClient>>) -> Json<AssetFull> {
    let asset = client.lock().await.get_asset_full(asset_id).unwrap();
    Json(asset)
}

#[post("/assets", format = "json", data = "<asset_ids>")]
pub async fn get_assets(asset_ids: Json<Vec<String>>, client: &State<Mutex<AssetsClient>>) -> Json<Vec<AssetBasic>> {
    let assets = client.lock().await.get_assets(asset_ids.0).unwrap();
    Json(assets)
}

#[post("/assets/add", format = "json", data = "<asset_id>")]
pub async fn add_asset(asset_id: Json<AssetId>, client: &State<Mutex<AssetsClient>>, assets_chain_provider: &State<Mutex<AssetsChainProvider>>) -> Json<Asset> {
    let asset_id = asset_id.0;

    let asset = assets_chain_provider
        .lock()
        .await
        .get_token_data(asset_id.chain, asset_id.token_id.clone().unwrap())
        .await
        .unwrap();
    client.lock().await.add_asset(asset.clone()).unwrap();
    Json(asset)
}

#[get("/assets/list")]
pub async fn get_assets_list(client: &State<Mutex<AssetsClient>>) -> Json<Vec<AssetBasic>> {
    let assets = client.lock().await.get_assets_list().unwrap();
    Json(assets)
}

#[get("/assets/search?<query>&<chains>&<limit>&<offset>")]
pub async fn get_assets_search(
    query: String,
    chains: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    client: &State<Mutex<AssetsSearchClient>>,
) -> Json<Vec<AssetBasic>> {
    let chains = chains.unwrap_or_default().split(',').flat_map(Chain::from_str).map(|x| x.to_string()).collect();
    let assets = client
        .lock()
        .await
        .get_assets_search(query.as_str(), chains, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .unwrap();
    Json(assets)
}

//TODO: Delete in favor of get_assets_by_device_id
#[get("/assets/by_device_id/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_ids_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<String>> {
    let assets = client.lock().await.get_assets_by_device_id(device_id, wallet_index, from_timestamp).unwrap();
    Json(assets)
}

#[get("/assets/device/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<String>> {
    let assets = client.lock().await.get_assets_by_device_id(device_id, wallet_index, from_timestamp).unwrap();
    Json(assets)
}
