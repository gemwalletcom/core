extern crate rocket;

use primitives::{NFTCollectible, NFTCollection};
use rocket::{response::status::NotFound, serde::json::Json, tokio::sync::Mutex, State};

use crate::{nft_client::NFTClient, response::ResponseResults};

#[get("/nft/<device_id>?<wallet_index>")]
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

#[get("/nft/<device_id>/<collection_id>?<wallet_index>")]
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
