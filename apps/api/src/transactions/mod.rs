pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::TransactionsClient;
use primitives::Transaction;
use primitives::TransactionsFetchOption;
use primitives::TransactionsResponse;
use rocket::{State, get, tokio::sync::Mutex};

#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v1(
    device_id: &str,
    wallet_index: i32,
    asset_id: Option<&str>,
    from_timestamp: Option<u32>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    let options: TransactionsFetchOption = TransactionsFetchOption {
        wallet_index,
        asset_id: asset_id.map(|s| s.to_string()),
        from_timestamp,
    };
    Ok(client.lock().await.get_transactions_by_device_id(device_id, options)?.transactions.into())
}

#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v2(
    device_id: &str,
    wallet_index: i32,
    asset_id: Option<&str>,
    from_timestamp: Option<u32>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    let options: TransactionsFetchOption = TransactionsFetchOption {
        wallet_index,
        asset_id: asset_id.map(|s| s.to_string()),
        from_timestamp,
    };
    Ok(client.lock().await.get_transactions_by_device_id(device_id, options)?.into())
}

#[get("/transactions/<id>")]
pub async fn get_transaction_by_id(id: &str, client: &State<Mutex<TransactionsClient>>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.lock().await.get_transaction_by_id(id)?.into())
}
