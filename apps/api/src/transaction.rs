extern crate rocket;

use crate::TransactionsClient;
use primitives::Transaction;
use primitives::TransactionsFetchOption;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/transactions/by_device_id/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id(
    device_id: &str,
    wallet_index: i32,
    asset_id: Option<String>,
    from_timestamp: Option<u32>,
    client: &State<Mutex<TransactionsClient>>,
) -> Json<Vec<Transaction>> {
    let options: TransactionsFetchOption = TransactionsFetchOption {
        wallet_index,
        asset_id,
        from_timestamp,
    };
    let transactions = client.lock().await.get_transactions_by_device_id(device_id, options).unwrap();
    Json(transactions)
}

#[get("/transactions/by_hash/<hash>")]
pub async fn get_transactions_by_hash(hash: &str, client: &State<Mutex<TransactionsClient>>) -> Json<Vec<Transaction>> {
    let transactions = client.lock().await.get_transactions_by_hash(hash).unwrap();
    Json(transactions)
}
