use rocket::{get, tokio::sync::Mutex, State};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::Asset;

use super::ChainClient;

#[get("/chain/token/<chain>/<token_id>")]
pub async fn get_token(chain: ChainParam, token_id: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Asset>, ApiError> {
    Ok(client.lock().await.get_token_data(chain.0, token_id.to_string()).await?.into())
}
