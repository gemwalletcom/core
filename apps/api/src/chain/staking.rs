use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::State;
use std::str::FromStr;

use primitives::chain::Chain;
use primitives::StakeValidator;
use settings_chain::ChainProviders;

#[get("/chain/staking/validators/<chain>")]
pub async fn get_validators(chain: String, chain_providers: &State<Mutex<ChainProviders>>) -> Result<Json<Vec<StakeValidator>>, rocket::http::Status> {
    let chain = Chain::from_str(&chain).map_err(|_| rocket::http::Status::BadRequest)?;

    if !chain.is_stake_supported() {
        return Err(rocket::http::Status::BadRequest);
    }

    let chain_providers = chain_providers.lock().await;
    let validators = chain_providers.get_validators(chain).await.unwrap();

    Ok(Json(validators))
}
