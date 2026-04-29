use chrono::Utc;
use primitives::{Chain, TransactionState, TransactionStateRequest, TransactionUpdate, chain_transaction_timeout};
use std::sync::Arc;

use crate::gateway::ChainClientFactory;

use super::TransactionStatusError;

pub struct TransactionStatusClient {
    chain_factory: Arc<ChainClientFactory>,
}

impl TransactionStatusClient {
    pub fn new(chain_factory: Arc<ChainClientFactory>) -> Self {
        Self { chain_factory }
    }

    pub async fn status(&self, chain: Chain, request: TransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let elapsed = Utc::now().timestamp().saturating_sub(request.created_at).max(0) as u64;
        let timeout = chain_transaction_timeout(chain) as u64 / 1000;

        let provider = self.chain_factory.create(chain).await?;
        match provider.get_transaction_status(request).await {
            Ok(update) => Ok(apply_timeout(update, elapsed, timeout)),
            Err(_) if elapsed > timeout => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
            Err(err) => Err(TransactionStatusError::NetworkError(err.to_string())),
        }
    }
}

fn apply_timeout(update: TransactionUpdate, elapsed_secs: u64, timeout_secs: u64) -> TransactionUpdate {
    let still_running = match update.state {
        TransactionState::Pending | TransactionState::InTransit => true,
        TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => false,
    };
    if still_running && elapsed_secs > timeout_secs {
        TransactionUpdate::new_state(TransactionState::Failed)
    } else {
        update
    }
}
