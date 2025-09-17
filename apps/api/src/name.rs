use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use name_resolver::client::Client as NameClient;
use primitives::name::NameRecord;
use rocket::{get, tokio::sync::Mutex, State};

#[get("/name/resolve/<name>?<chain>")]
pub async fn get_name_resolve(name: &str, chain: ChainParam, name_client: &State<Mutex<NameClient>>) -> Result<ApiResponse<NameRecord>, ApiError> {
    Ok(name_client.lock().await.resolve(name, chain.0).await?.into())
}
