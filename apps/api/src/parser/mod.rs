mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::ParserClient;
use primitives::{Chain, Transaction};
use rocket::{get, tokio::sync::Mutex, State};
use std::str::FromStr;

#[get("/parser/chains/<chain>/blocks/<block_number>?<transaction_type>")]
pub async fn get_parser_block(
    chain: &str,
    block_number: i64,
    transaction_type: Option<&str>,
    parser_client: &State<Mutex<ParserClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    let chain = Chain::from_str(chain).map_err(|_| ApiError::BadRequest("Invalid chain".to_string()))?;
    Ok(parser_client.lock().await.get_block(chain, block_number, transaction_type).await?.into())
}

#[get("/parser/chains/<chain>/blocks/<block_number>/finalize?<address>&<transaction_type>")]
pub async fn get_parser_block_finalize(
    chain: &str,
    block_number: i64,
    address: &str,
    transaction_type: Option<&str>,
    parser_client: &State<Mutex<ParserClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    let chain = Chain::from_str(chain).map_err(|_| ApiError::BadRequest("Invalid chain".to_string()))?;
    Ok(parser_client
        .lock()
        .await
        .get_block_finalize(chain, block_number, vec![address.to_string()], transaction_type)
        .await?
        .into())
}

#[get("/parser/chains/<chain>")]
pub async fn get_parser_block_number_latest(chain: &str, parser_client: &State<Mutex<ParserClient>>) -> Result<ApiResponse<i64>, ApiError> {
    let chain = Chain::from_str(chain).map_err(|_| ApiError::BadRequest("Invalid chain".to_string()))?;
    Ok(parser_client.lock().await.get_block_number_latest(chain).await?.into())
}
