use rocket::{State, get, tokio::sync::Mutex};

use crate::params::{AddressParam, ChainParam};
use crate::responders::{ApiError, ApiResponse};
use primitives::{AddressBalances, ChainAddress};

use super::ChainClient;

#[get("/chain/balances/<chain>/<address>")]
pub async fn get_balances(chain: ChainParam, address: AddressParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<AddressBalances>, ApiError> {
    let request = ChainAddress::new(chain.0, address.0);
    let client = client.lock().await;
    let coin = client.get_balances_coin(request.clone()).await?;
    let staking = client.get_balances_staking(request.clone()).await?;
    let assets = client.get_balances_assets(request).await?;
    Ok(AddressBalances { coin, staking, assets }.into())
}
