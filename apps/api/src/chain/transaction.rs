use rocket::{get, tokio::sync::Mutex, State};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{ChainAddress, Transaction};

use super::ChainClient;

#[get("/chain/transactions/<chain>/<address>")]
pub async fn get_transactions(chain: ChainParam, address: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.to_string());
    Ok(client.lock().await.get_transactions(request).await?.into())
}
