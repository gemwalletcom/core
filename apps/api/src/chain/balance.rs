use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::str::FromStr;

use primitives::{AssetBalance, Chain, ChainAddress};

use super::ChainClient;

#[get("/chain/balances/<chain>/<address>")]
pub async fn get_balances(chain: String, address: String, chain_client: &State<Mutex<ChainClient>>) -> Json<Vec<AssetBalance>> {
    let chain = Chain::from_str(&chain).unwrap();
    let request = ChainAddress::new(chain, address);
    let balances = chain_client.lock().await.get_balances(request).await.unwrap();
    Json(balances)
}
