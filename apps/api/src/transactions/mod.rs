pub mod client;
use crate::params::{DeviceIdParam, TransactionIdParam};
use crate::responders::{ApiError, ApiResponse};
pub use client::TransactionsClient;
use primitives::{Transaction, TransactionsResponse};
use rocket::{State, get, tokio::sync::Mutex};

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/wallets/<wallet_id>/transactions
#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v1(
    device_id: DeviceIdParam,
    wallet_index: i32,
    asset_id: Option<&str>,
    from_timestamp: Option<u64>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client
        .lock()
        .await
        .get_transactions_by_device_id(&device_id.0, wallet_index, asset_id.map(|s| s.to_string()), from_timestamp)?
        .transactions
        .into())
}

// TODO: Remove once all clients migrate to /v1/devices/<device_id>/wallets/<wallet_id>/transactions
#[get("/transactions/device/<device_id>?<wallet_index>&<asset_id>&<from_timestamp>")]
pub async fn get_transactions_by_device_id_v2(
    device_id: DeviceIdParam,
    wallet_index: i32,
    asset_id: Option<&str>,
    from_timestamp: Option<u64>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client
        .lock()
        .await
        .get_transactions_by_device_id(&device_id.0, wallet_index, asset_id.map(|s| s.to_string()), from_timestamp)?
        .into())
}

#[get("/transactions/<id>")]
pub async fn get_transaction_by_id(id: TransactionIdParam, client: &State<Mutex<TransactionsClient>>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.lock().await.get_transaction_by_id(&id.0)?.into())
}
