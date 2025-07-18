pub mod client;
pub use client::TransactionsClient;
extern crate rocket;

use primitives::Transaction;
use primitives::TransactionsFetchOption;
use primitives::TransactionsResponse;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;

#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v1(
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
    Json(client.lock().await.get_transactions_by_device_id(device_id, options).unwrap().transactions)
}

#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v2(
    device_id: &str,
    wallet_index: i32,
    asset_id: Option<String>,
    from_timestamp: Option<u32>,
    client: &State<Mutex<TransactionsClient>>,
) -> Json<TransactionsResponse> {
    let options: TransactionsFetchOption = TransactionsFetchOption {
        wallet_index,
        asset_id,
        from_timestamp,
    };
    Json(client.lock().await.get_transactions_by_device_id(device_id, options).unwrap())
}

#[get("/transactions/<id>")]
pub async fn get_transactions_by_id(id: &str, client: &State<Mutex<TransactionsClient>>) -> Json<Vec<Transaction>> {
    Json(client.lock().await.get_transactions_by_id(id).unwrap())
}
