use primitives::{response::ResponseResultOld, ScanAddress, ScanTransaction, ScanTransactionPayload};
use rocket::{get, post, serde::json::Json, tokio::sync::Mutex, State};

pub mod client;
pub mod providers;

pub use client::ScanClient;
pub use providers::ScanProviderFactory;

#[post("/scan/transaction", data = "<request>")]
pub async fn scan_transaction(request: Json<ScanTransactionPayload>, client: &State<Mutex<ScanClient>>) -> Json<ResponseResultOld<ScanTransaction>> {
    let result: ScanTransaction = client.lock().await.get_scan_transaction(request.0).await.unwrap();
    Json(ResponseResultOld::new(result))
}

#[get("/scan/address/<address>")]
pub async fn get_scan_address(address: String, client: &State<Mutex<ScanClient>>) -> Json<ResponseResultOld<Vec<ScanAddress>>> {
    let scan_addresses = client.lock().await.get_scan_address(&address).await.unwrap();
    Json(ResponseResultOld::new(scan_addresses))
}
