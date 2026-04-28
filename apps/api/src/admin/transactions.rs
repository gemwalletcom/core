use primitives::{Transaction, TransactionId};
use rocket::serde::json::Json;
use rocket::{State, post, tokio::sync::Mutex};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

use crate::admin::AdminAuthorized;
use crate::chain::ChainClient;
use crate::responders::{ApiError, ApiResponse};

#[post("/transactions/add", format = "json", data = "<transaction_id>")]
pub async fn add_transaction(
    _admin: AdminAuthorized,
    transaction_id: Json<TransactionId>,
    chain_client: &State<Mutex<ChainClient>>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    let client = chain_client.lock().await;

    let transaction_id = transaction_id.0;
    let transaction = client.get_transaction_by_hash(transaction_id.chain, transaction_id.hash).await?;

    if let Some(transaction) = transaction.as_ref() {
        let payload = TransactionsPayload::new(transaction.asset_id.chain, vec![transaction.clone()]);
        stream_producer.publish_transactions(payload).await?;
    }

    Ok(transaction.into())
}
