pub mod client;
use crate::params::TransactionIdParam;
use crate::responders::{ApiError, ApiResponse};
pub use client::TransactionsClient;
use primitives::Transaction;
use rocket::{State, get, tokio::sync::Mutex};

#[get("/transactions/<id>")]
pub async fn get_transaction_by_id(id: TransactionIdParam, client: &State<Mutex<TransactionsClient>>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.lock().await.get_transaction_by_id(&id.0)?.into())
}
