pub mod cilent;
pub use cilent::{AssetsChainProvider, AssetsClient, AssetsSearchClient};
use rocket::http::Status;
extern crate rocket;

use std::str::FromStr;

use primitives::{Asset, AssetBalance, AssetBasic, AssetFull, AssetId, Chain, ChainAddress, Transaction};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

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
    assets_chain_provider: &State<Mutex<AssetsChainProvider>>,
) -> Json<Vec<Asset>> {
    let asset_id = asset_id.0.first().unwrap();

    let asset = assets_chain_provider
        .lock()
        .await
        .get_token_data(asset_id.chain, asset_id.token_id.clone().unwrap())
        .await
        .unwrap();
    client.lock().await.add_assets(vec![asset.clone()]).unwrap();

    Json(vec![asset])
}

#[get("/assets/list")]
pub async fn get_assets_list(client: &State<Mutex<AssetsClient>>) -> Json<Vec<AssetBasic>> {
    let assets = client.lock().await.get_assets_list().unwrap();
    Json(assets)
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

#[post("/assets/balances", format = "json", data = "<requests>")]
pub async fn get_assets_balances(
    requests: Json<Vec<ChainAddress>>,
    assets_chain_provider: &State<Mutex<AssetsChainProvider>>,
) -> Result<Json<Vec<AssetBalance>>, Status> {
    match assets_chain_provider.lock().await.get_assets_balances(requests.0).await {
        Ok(assets) => Ok(Json(assets)),
        Err(error) => {
            println!("get_assets_balances error: {error:?}");
            Err(Status::InternalServerError)
        }
    }
}

#[post("/assets/transactions", format = "json", data = "<requests>")]
pub async fn get_assets_transactions(
    requests: Json<Vec<ChainAddress>>,
    assets_chain_provider: &State<Mutex<AssetsChainProvider>>,
) -> Result<Json<Vec<Transaction>>, Status> {
    match assets_chain_provider.lock().await.get_assets_transactions(requests.0).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(error) => {
            println!("get_assets_transactions error: {error:?}");
            Err(Status::InternalServerError)
        }
    }
}
