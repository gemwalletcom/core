use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
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
