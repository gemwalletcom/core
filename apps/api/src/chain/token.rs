use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::str::FromStr;

use primitives::{Asset, Chain};

use super::ChainClient;

#[get("/chain/token/<chain>/<token_id>")]
pub async fn get_token(chain: String, token_id: String, chain_client: &State<Mutex<ChainClient>>) -> Json<Asset> {
    let chain = Chain::from_str(&chain).unwrap();
    let asset = chain_client.lock().await.get_token_data(chain, token_id).await.unwrap();
    Json(asset)
}
