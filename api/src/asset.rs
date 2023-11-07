extern crate rocket;
use primitives::AssetFull;
use rocket::serde::json::Json;
use crate::AssetsClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/assets/<asset_id>")]
pub async fn get_asset(
    asset_id: &str,
    client: &State<Mutex<AssetsClient>>,
) -> Json<AssetFull> {
    let asset = client.lock().await.get_asset_full(asset_id).unwrap();
    Json(asset)
}

#[get("/assets/search?<query>")]
pub async fn get_assets_search(
    query: String,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<AssetFull>> {
    let assets = client.lock().await.get_assets_search(query.as_str()).unwrap();
    Json(assets)
}

#[get("/assets/by_device_id/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_ids_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<String>> {
    let assets = client.lock().await.get_assets_ids_by_device_id(device_id, wallet_index, from_timestamp).unwrap();
    Json(assets)
}