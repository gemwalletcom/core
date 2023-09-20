extern crate rocket;
use primitives::Transaction;
use rocket::serde::json::Json;
use crate::TransactionsClient;
use rocket::State;
use rocket::tokio::sync::Mutex;

#[get("/transactions/<device_id>")]
pub async fn get_transactions_by_device_id(
    device_id: &str,
    client: &State<Mutex<TransactionsClient>>,
) -> Json<Vec<Transaction>> {
    let transactions = client.lock().await.get_transactions_by_device_id(device_id).unwrap();
    Json(transactions)
}