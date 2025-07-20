use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::str::FromStr;

use primitives::{Chain, ChainAddress, Transaction};

use super::ChainClient;

#[get("/chain/transactions/<chain>/<address>")]
pub async fn get_transactions(chain: String, address: String, chain_client: &State<Mutex<ChainClient>>) -> Json<Vec<Transaction>> {
    let chain = Chain::from_str(&chain).unwrap();
    let request = ChainAddress::new(chain, address);
    let transactions = chain_client.lock().await.get_transactions(request).await.unwrap();
    Json(transactions)
}
