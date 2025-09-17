use rocket::{get, tokio::sync::Mutex, State};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::StakeValidator;

use super::ChainClient;

#[get("/chain/staking/validators/<chain>")]
pub async fn get_validators(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<StakeValidator>>, ApiError> {
    Ok(client.lock().await.get_validators(chain.0).await?.into())
}

#[get("/chain/staking/apy/<chain>")]
pub async fn get_staking_apy(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<f64>, ApiError> {
    Ok(client.lock().await.get_staking_apy(chain.0).await?.into())
}
