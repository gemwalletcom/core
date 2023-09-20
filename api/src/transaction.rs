extern crate rocket;

use primitives::Transaction;
use rocket::serde::json::Json;
use primitives::TransactionsFetchOption;
use crate::TransactionsClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/transactions/by_device_id/<device_id>?<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id(
    device_id: &str,
    asset_id: Option<&str>,
    from_timestamp: Option<i64>,
    client: &State<Mutex<TransactionsClient>>,
) -> Json<Vec<Transaction>> {
    let options: TransactionsFetchOption<'_> = TransactionsFetchOption{
        asset_id,
        from_timestamp,
    };
    let transactions = client.lock().await.get_transactions_by_device_id(device_id, options).unwrap();
    Json(transactions)
}

#[get("/transactions/by_hash/<hash>")]
pub async fn get_transactions_by_hash(
    hash: &str,
    client: &State<Mutex<TransactionsClient>>,
) -> Json<Vec<Transaction>> {
    let transactions = client.lock().await.get_transactions_by_hash(hash).unwrap();
    Json(transactions)
}