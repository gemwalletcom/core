use rocket::{State, get, tokio::sync::Mutex};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{StakeLockTime, StakeValidator};

use super::ChainClient;

#[get("/chain/staking/validators/<chain>")]
pub async fn get_validators(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<StakeValidator>>, ApiError> {
    Ok(client.lock().await.get_validators(chain.0).await?.into())
}

#[get("/chain/staking/apy/<chain>")]
pub async fn get_staking_apy(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<f64>, ApiError> {
    Ok(client.lock().await.get_staking_apy(chain.0).await?.into())
}

#[get("/chain/staking/lock-time/<chain>")]
pub async fn get_staking_lock_time(chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<StakeLockTime>, ApiError> {
    Ok(client.lock().await.get_staking_lock_time(chain.0).await?.into())
}
