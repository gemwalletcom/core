pub mod cilent;
pub use cilent::{AssetsClient, AssetsSearchClient};
use rocket::{get, http::Status, post, serde::json::Json, tokio::sync::Mutex, State};

use std::str::FromStr;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, Chain};

#[get("/assets/<asset_id>")]
pub async fn get_asset(asset_id: &str, client: &State<Mutex<AssetsClient>>) -> Result<Json<AssetFull>, Status> {
    let result = client.lock().await.get_asset_full(asset_id);
    match result {
        Ok(asset) => Ok(Json(asset)),
        Err(error) => {
            println!("get_asset error: {asset_id}, {error:?}");
            Err(Status::NotFound)
        }
    }
}

#[post("/assets", format = "json", data = "<asset_ids>")]
pub async fn get_assets(asset_ids: Json<Vec<String>>, client: &State<Mutex<AssetsClient>>) -> Json<Vec<AssetBasic>> {
    let assets = client.lock().await.get_assets(asset_ids.0).unwrap();
    Json(assets)
}

#[post("/assets/add", format = "json", data = "<asset_id>")]
pub async fn add_asset(
    asset_id: Json<Vec<AssetId>>,
    client: &State<Mutex<AssetsClient>>,
    chain_client: &State<Mutex<crate::chain::ChainClient>>,
) -> Json<Vec<Asset>> {
    let asset_id = asset_id.0.first().unwrap();

    let asset = chain_client
        .lock()
        .await
        .get_token_data(asset_id.chain, asset_id.token_id.clone().unwrap())
        .await
        .unwrap();
    client.lock().await.add_assets(vec![asset.clone()]).unwrap();

    Json(vec![asset])
}

#[get("/assets/search?<query>&<chains>&<tags>&<limit>&<offset>")]
pub async fn get_assets_search(
    query: String,
    chains: Option<String>,
    tags: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    client: &State<Mutex<AssetsSearchClient>>,
) -> Result<Json<Vec<AssetBasic>>, Status> {
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

    let assets = client
        .lock()
        .await
        .get_assets_search(query.as_str(), chains.clone(), tags.clone(), limit.unwrap_or(50), offset.unwrap_or(0))
        .await;
    match assets {
        Ok(assets) => Ok(Json(assets)),
        Err(error) => {
            println!("get_assets_search, query: {query:?}, tags: {tags:?}, chains: {chains:?} error: {error:?}");
            Err(Status::InternalServerError)
        }
    }
}

#[get("/assets/device/<device_id>?<wallet_index>&<from_timestamp>")]
pub async fn get_assets_by_device_id(
    device_id: &str,
    wallet_index: i32,
    from_timestamp: Option<u32>,
    client: &State<Mutex<AssetsClient>>,
) -> Json<Vec<AssetId>> {
    let assets = client.lock().await.get_assets_by_device_id(device_id, wallet_index, from_timestamp).unwrap();
    Json(assets)
}
