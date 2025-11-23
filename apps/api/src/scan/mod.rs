use crate::responders::{ApiError, ApiResponse};
use primitives::{ScanAddress, ScanTransaction, ScanTransactionPayload, response::ResponseResultNew};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

pub mod client;
pub mod providers;

pub use client::ScanClient;
pub use providers::ScanProviderFactory;

#[post("/scan/transaction", data = "<request>")]
pub async fn scan_transaction(
    request: Json<ScanTransactionPayload>,
    client: &State<Mutex<ScanClient>>,
) -> Result<ApiResponse<ResponseResultNew<ScanTransaction>>, ApiError> {
    let result: ScanTransaction = client.lock().await.get_scan_transaction(request.0).await?;
    Ok(ResponseResultNew::new(result).into())
}

#[get("/scan/address/<address>")]
pub async fn get_scan_address(address: &str, client: &State<Mutex<ScanClient>>) -> Result<ApiResponse<ResponseResultNew<Vec<ScanAddress>>>, ApiError> {
    let scan_addresses = client.lock().await.get_scan_address(address).await?;
    Ok(ResponseResultNew::new(scan_addresses).into())
}
