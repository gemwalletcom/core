use async_trait::async_trait;
use primitives::{TransactionStateInput, TransactionUpdate};

use crate::error::StateError;

#[async_trait]
pub trait ChainStateSource: Send + Sync {
    async fn get_transaction_status(&self, input: &TransactionStateInput) -> Result<TransactionUpdate, StateError>;
}

#[async_trait]
pub trait TransactionUpdateSink: Send + Sync {
    async fn on_update(&self, id: String, update: TransactionUpdate);
}
