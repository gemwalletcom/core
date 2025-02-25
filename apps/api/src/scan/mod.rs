extern crate rocket;
use primitives::{ScanTransaction, ScanTransactionPayload};
use rocket::{serde::json::Json, tokio::sync::Mutex, State};

pub mod client;
pub mod providers;

pub use client::ScanClient;
pub use providers::ScanProviderFactory;

#[post("/scan/transaction", data = "<request>")]
pub async fn scan_transaction(request: Json<ScanTransactionPayload>, client: &State<Mutex<ScanClient>>) -> Json<ScanTransaction> {
    let result = client.lock().await.get_scan_transaction(request.0).await.unwrap();
    Json(result)
}
