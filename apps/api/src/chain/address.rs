use rocket::{State, get, tokio::sync::Mutex};

use crate::params::{AddressParam, ChainParam};
use crate::responders::{ApiError, ApiResponse};
use primitives::{AddressBalances, AssetBalance, ChainAddress, Transaction};

use super::ChainClient;

#[get("/chain/<chain>/<address>/balances")]
pub async fn get_balances(chain: ChainParam, address: AddressParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<AddressBalances>, ApiError> {
    let request = ChainAddress::new(chain.0, address.0);
    let client = client.lock().await;
    let coin = client.get_balances_coin(request.clone()).await?;
    let staking = client.get_balances_staking(request.clone()).await?;
    let assets = client.get_balances_assets(request).await?;
    Ok(AddressBalances { coin, staking, assets }.into())
}

#[get("/chain/<chain>/<address>/assets")]
pub async fn get_assets(chain: ChainParam, address: AddressParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<AssetBalance>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.0);
    Ok(client.lock().await.get_balances_assets(request).await?.into())
}

#[get("/chain/<chain>/<address>/transactions")]
pub async fn get_transactions(chain: ChainParam, address: AddressParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    let request = ChainAddress::new(chain.0, address.0);
    Ok(client.lock().await.get_transactions(request).await?.into())
}
