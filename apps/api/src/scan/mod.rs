extern crate rocket;
use primitives::{ResponseResult, ScanTransaction, ScanTransactionPayload};
use rocket::{serde::json::Json, tokio::sync::Mutex, State};

pub mod client;
pub mod providers;

pub use client::ScanClient;
pub use providers::ScanProviderFactory;

#[post("/scan/transaction", data = "<request>")]
pub async fn scan_transaction(request: Json<ScanTransactionPayload>, client: &State<Mutex<ScanClient>>) -> Json<ResponseResult<ScanTransaction>> {
    let result: ScanTransaction = client.lock().await.get_scan_transaction(request.0).await.unwrap();
    Json(ResponseResult::new(result))
}
