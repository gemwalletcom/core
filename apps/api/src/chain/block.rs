use rocket::{State, get, tokio::sync::Mutex};

use crate::params::{AddressParam, ChainParam};
use crate::responders::{ApiError, ApiResponse};
use primitives::Transaction;

use super::ChainClient;

#[get("/chain/blocks/<chain>/latest")]
pub async fn get_latest_block_number(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<i64>, ApiError> {
    Ok(client.lock().await.get_latest_block(chain.0).await?.into())
}

#[get("/chain/blocks/<chain>/<block_number>?<transaction_type>")]
pub async fn get_block_transactions(
    chain: ChainParam,
    block_number: i64,
    transaction_type: Option<&str>,
    client: &State<Mutex<ChainClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.lock().await.get_block_transactions(chain.0, block_number, transaction_type).await?.into())
}

#[get("/chain/blocks/<chain>/<block_number>/finalize?<address>&<transaction_type>")]
pub async fn get_block_transactions_finalize(
    chain: ChainParam,
    block_number: i64,
    address: AddressParam,
    transaction_type: Option<&str>,
    client: &State<Mutex<ChainClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client
        .lock()
        .await
        .get_block_transactions_finalize(chain.0, block_number, vec![address.0], transaction_type)
        .await?
        .into())
}
