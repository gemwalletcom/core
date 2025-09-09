use rocket::{get, tokio::sync::Mutex, State};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{AssetBalance, ChainAddress};

use super::ChainClient;

#[get("/chain/balances/<chain>/<address>")]
pub async fn get_balances(chain: ChainParam, address: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<AssetBalance>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.to_string());
    Ok(client.lock().await.get_balances(request).await?.into())
}
