use rocket::serde::json::Json;
use rocket::{State, get, post, tokio::sync::Mutex};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{Transaction, TransactionId};

use super::ChainClient;

#[get("/chain/transactions/<chain>/<hash>")]
pub async fn get_transaction(chain: ChainParam, hash: &str, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    Ok(client.lock().await.get_transaction_by_hash(chain.0, hash.to_string()).await?.into())
}

#[post("/transactions/add", format = "json", data = "<transaction_id>")]
pub async fn add_transaction(
    transaction_id: Json<TransactionId>,
    chain_client: &State<Mutex<ChainClient>>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    let client = chain_client.lock().await;

    let transaction_id = transaction_id.0;
    let transaction = client.get_transaction_by_hash(transaction_id.chain, transaction_id.hash).await?;

    if let Some(transaction) = transaction.as_ref() {
        let payload = TransactionsPayload::new(transaction.asset_id.chain, vec![], vec![transaction.clone()]);
        stream_producer.publish_transactions(payload).await?;
    }

    Ok(transaction.into())
}
