mod client;
pub use client::ParserClient;

extern crate rocket;
use std::str::FromStr;

use primitives::{Chain, Transaction};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/parser/chains/<chain>/blocks/<block_number>?<transaction_type>")]
pub async fn get_parser_block(
    chain: &str,
    block_number: i64,
    transaction_type: Option<&str>,
    parser_client: &State<Mutex<ParserClient>>,
) -> Json<Vec<Transaction>> {
    let chain = Chain::from_str(chain).unwrap();
    let transactions = parser_client.lock().await.get_block(chain, block_number, transaction_type).await.unwrap();
    Json(transactions)
}

#[get("/parser/chains/<chain>/blocks/<block_number>/finalize?<address>&<transaction_type>")]
pub async fn get_parser_block_finalize(
    chain: &str,
    block_number: i64,
    address: &str,
    transaction_type: Option<&str>,
    parser_client: &State<Mutex<ParserClient>>,
) -> Json<Vec<Transaction>> {
    let chain = Chain::from_str(chain).unwrap();
    let transactions = parser_client
        .lock()
        .await
        .get_block_finalize(chain, block_number, vec![address.to_string()], transaction_type)
        .await
        .unwrap();
    Json(transactions)
}

#[get("/parser/chains/<chain>")]
pub async fn get_parser_block_number_latest(chain: &str, parser_client: &State<Mutex<ParserClient>>) -> Json<i64> {
    let chain = Chain::from_str(chain).unwrap();
    let block_number = parser_client.lock().await.get_block_number_latest(chain).await.unwrap();
    Json(block_number)
}
