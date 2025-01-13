extern crate rocket;
use primitives::{Chain, NFTCollection, ResponseResult};
use rocket::{response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};
use std::str::FromStr;

use crate::nft_client::NFTClient;

// by device

#[get("/nft/assets/<device_id>?<wallet_index>")]
pub async fn get_nft_assets(
    device_id: &str,
    wallet_index: i32,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResult<Vec<NFTCollection>>>, NotFound<String>> {
    let result = client.lock().await.get_nft_assets(device_id, wallet_index).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult { data })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

// by address. mostly for testing purposes

#[get("/nft/assets_by_chain/<chain>?<address>")]
pub async fn get_nft_assets_by_chain(
    chain: &str,
    address: &str,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResult<Vec<NFTCollection>>>, NotFound<String>> {
    let chain = Chain::from_str(chain).unwrap();
    let result = client.lock().await.get_nft_assets_by_chain(chain, address).await;
    match result {
        Ok(data) => Ok(Json(ResponseResult { data })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
