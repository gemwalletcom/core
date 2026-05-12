use rocket::{State, get, tokio::sync::Mutex};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{Transaction, TransactionUpdate};

use super::ChainClient;

#[get("/chain/transactions/<chain>/<hash>")]
pub async fn get_transaction(chain: ChainParam, hash: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    Ok(client.lock().await.get_transaction_by_hash(chain.0, hash.to_string()).await?.into())
}

#[get("/chain/transactions/<chain>/<hash>/status")]
pub async fn get_transaction_status(chain: ChainParam, hash: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<TransactionUpdate>, ApiError> {
    Ok(client.lock().await.get_transaction_status(chain.0, hash.to_string()).await?.into())
}
