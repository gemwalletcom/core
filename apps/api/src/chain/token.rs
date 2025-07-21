use rocket::{get, serde::json::Json, tokio::sync::Mutex, State};
use std::str::FromStr;

use primitives::{Asset, Chain};

use super::ChainClient;

#[get("/chain/token/<chain>/<token_id>")]
pub async fn get_token(chain: String, token_id: String, chain_client: &State<Mutex<ChainClient>>) -> Json<Asset> {
    let chain = Chain::from_str(&chain).unwrap();
    let asset = chain_client.lock().await.get_token_data(chain, token_id).await.unwrap();
    Json(asset)
}
