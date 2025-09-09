use crate::responders::{ApiError, ApiResponse};
use primitives::{response::ResponseResultOld, ScanAddress, ScanTransaction, ScanTransactionPayload};
use rocket::{get, post, serde::json::Json, tokio::sync::Mutex, State};

pub mod client;
pub mod providers;

pub use client::ScanClient;
pub use providers::ScanProviderFactory;

#[post("/scan/transaction", data = "<request>")]
pub async fn scan_transaction(
    request: Json<ScanTransactionPayload>,
    client: &State<Mutex<ScanClient>>,
) -> Result<ApiResponse<ResponseResultOld<ScanTransaction>>, ApiError> {
    let result: ScanTransaction = client.lock().await.get_scan_transaction(request.0).await?;
    Ok(ResponseResultOld::new(result).into())
}

#[get("/scan/address/<address>")]
pub async fn get_scan_address(address: String, client: &State<Mutex<ScanClient>>) -> Result<ApiResponse<ResponseResultOld<Vec<ScanAddress>>>, ApiError> {
    let scan_addresses = client.lock().await.get_scan_address(&address).await?;
    Ok(ResponseResultOld::new(scan_addresses).into())
}
