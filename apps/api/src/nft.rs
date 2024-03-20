extern crate rocket;
use primitives::{Chain, NFTCollectible, NFTCollection};
use rocket::{response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};
use std::str::FromStr;

use crate::{nft_client::NFTClient, response::ResponseResults};

// by device

#[get("/nft/collectibles_by_/<device_id>?<wallet_index>")]
pub async fn get_nft_collections(
    device_id: &str,
    wallet_index: i32,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResults<NFTCollection>>, NotFound<String>> {
    let result = client
        .lock()
        .await
        .get_nft_collections(device_id, wallet_index)
        .await;
    match result {
        Ok(results) => Ok(Json(ResponseResults { results })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

#[get("/nft/collectibles_by_device_id/<device_id>/<collection_id>?<wallet_index>")]
pub async fn get_nft_collectibles(
    device_id: &str,
    collection_id: &str,
    wallet_index: i32,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResults<NFTCollectible>>, NotFound<String>> {
    let result = client
        .lock()
        .await
        .get_nft_collectibles(device_id, collection_id, wallet_index)
        .await;
    match result {
        Ok(results) => Ok(Json(ResponseResults { results })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

// by address. mostly for testing purposes

#[get("/nft/collections_by_chain_address/<chain>?<address>")]
pub async fn get_nft_collections_by_chain_address(
    chain: &str,
    address: &str,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResults<NFTCollection>>, NotFound<String>> {
    let chain = Chain::from_str(chain).unwrap();
    let result = client
        .lock()
        .await
        .get_nft_collections_by_address(chain, address)
        .await;
    match result {
        Ok(results) => Ok(Json(ResponseResults { results })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}

#[get("/nft/collectibles_by_chain_address/<chain>/<collection_id>?<address>")]
pub async fn get_nft_collectibles_by_chain_address(
    chain: &str,
    collection_id: &str,
    address: &str,
    client: &State<Mutex<NFTClient>>,
) -> Result<Json<ResponseResults<NFTCollectible>>, NotFound<String>> {
    let chain = Chain::from_str(chain).unwrap();
    let result = client
        .lock()
        .await
        .get_nft_collectibles_by_address(chain, collection_id, address)
        .await;
    match result {
        Ok(results) => Ok(Json(ResponseResults { results })),
        Err(err) => Err(NotFound(err.to_string())),
    }
}
