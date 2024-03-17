extern crate rocket;
use std::str::FromStr;

use crate::AssetsClient;
use primitives::{AssetFull, Chain};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/assets/<asset_id>")]
pub async fn get_asset(asset_id: &str, client: &State<Mutex<AssetsClient>>) -> Json<AssetFull> {
    let asset = client.lock().await.get_asset_full(asset_id).unwrap();
    Json(asset)
}

#[get("/assets/search?<query>&<chains>&<limit>&<offset>")]
pub async fn get_assets_search(
    query: String,
    chains: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<AssetFull>> {
    let chains = chains
        .unwrap_or_default()
        .split(',')
        .flat_map(Chain::from_str)
        .map(|x| x.to_string())
        .collect();
    let assets = client
        .lock()
        .await
        .get_assets_search(
            query.as_str(),
            chains,
            limit.unwrap_or(50),
            offset.unwrap_or(0),
        )
        .unwrap();
    Json(assets)
}

#[get("/assets/by_device_id/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_ids_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<String>> {
    let assets = client
        .lock()
        .await
        .get_assets_ids_by_device_id(device_id, wallet_index, from_timestamp)
        .unwrap();
    Json(assets)
}
