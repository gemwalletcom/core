extern crate rocket;
use primitives::{AssetInfos, Asset};
use rocket::serde::json::Json;
use crate::AssetsClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/assets/<asset_id>")]
pub async fn get_asset(
    asset_id: &str,
    client: &State<Mutex<AssetsClient>>,
) -> Json<AssetInfos> {
    let asset = client.lock().await.get_asset(asset_id).unwrap();
    Json(AssetInfos { asset, info: None })
}

#[get("/assets/search?<query>")]
pub async fn get_assets_search(
    query: String,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<Asset>> {
    let assets = client.lock().await.get_assets_search(query.as_str()).unwrap();
    Json(assets)
}