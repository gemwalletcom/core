use rocket::{get, serde::json::Json, tokio::sync::Mutex, State};
use std::str::FromStr;

use primitives::chain::Chain;
use primitives::StakeValidator;

use super::ChainClient;

#[get("/chain/staking/validators/<chain>")]
pub async fn get_validators(chain: String, chain_client: &State<Mutex<ChainClient>>) -> Json<Vec<StakeValidator>> {
    let chain = Chain::from_str(&chain).unwrap();
    let validators = chain_client.lock().await.get_validators(chain).await.unwrap();
    Json(validators)
}

#[get("/chain/staking/apy/<chain>")]
pub async fn get_staking_apy(chain: String, chain_client: &State<Mutex<ChainClient>>) -> Json<f64> {
    let chain = Chain::from_str(&chain).unwrap();
    let apy = chain_client.lock().await.get_staking_apy(chain).await.unwrap();
    Json(apy)
}
