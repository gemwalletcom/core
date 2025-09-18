use rocket::{get, tokio::sync::Mutex, State};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{AssetBalance, ChainAddress};

use super::ChainClient;

#[get("/chain/balances/<chain>/<address>/coin")]
pub async fn get_balances_coin(chain: ChainParam, address: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<AssetBalance>, ApiError> {
    let request = ChainAddress::new(chain.0, address.to_string());
    Ok(client.lock().await.get_balances_coin(request).await?.into())
}

#[get("/chain/balances/<chain>/<address>/assets")]
pub async fn get_balances_assets(chain: ChainParam, address: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<AssetBalance>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.to_string());
    Ok(client.lock().await.get_balances_assets(request).await?.into())
}

#[get("/chain/balances/<chain>/<address>/staking")]
pub async fn get_balances_staking(chain: ChainParam, address: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Option<AssetBalance>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.to_string());
    Ok(client.lock().await.get_balances_staking(request).await?.into())
}
