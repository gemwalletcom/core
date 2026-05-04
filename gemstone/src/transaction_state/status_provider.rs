use chrono::Utc;
use primitives::{Chain, TransactionChange, TransactionMetadata, TransactionState, TransactionUpdate, chain_transaction_timeout};
use std::sync::Arc;
use swapper::{SwapperProvider, swapper::GemSwapper};

use crate::gateway::ChainClientFactory;
use crate::models::GemTransactionStateRequest;

use super::TransactionStatusError;

pub struct StatusProvider {
    chain_factory: Arc<ChainClientFactory>,
    swapper: GemSwapper,
}

impl StatusProvider {
    pub fn new(chain_factory: Arc<ChainClientFactory>, swapper: GemSwapper) -> Self {
        Self { chain_factory, swapper }
    }

    pub async fn get(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let created_at = request.created_at;
        let result = match request.swap_provider {
            Some(provider) => self.get_swap_status(chain, provider, &request.id).await,
            None => self.get_chain_status(chain, request).await,
        };
        get_transaction_update(chain, created_at, result)
    }

    async fn get_chain_status(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let provider = self.chain_factory.create(chain).await?;
        provider
            .get_transaction_status(request.into())
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))
    }

    async fn get_swap_status(&self, chain: Chain, provider: SwapperProvider, transaction_hash: &str) -> Result<TransactionUpdate, TransactionStatusError> {
        let result = self
            .swapper
            .get_swap_result(chain, provider, transaction_hash)
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))?;

        let state = result.status.transaction_state().unwrap_or(TransactionState::InTransit);
        let changes = result
            .metadata
            .map(|m| vec![TransactionChange::Metadata(TransactionMetadata::Swap(m))])
            .unwrap_or_default();
        Ok(TransactionUpdate::new(state, changes))
    }
}

fn get_transaction_update(
    chain: Chain,
    created_at: i64,
    result: Result<TransactionUpdate, TransactionStatusError>,
) -> Result<TransactionUpdate, TransactionStatusError> {
    let elapsed = Utc::now().timestamp_millis() - created_at;
    let timeout = i64::from(chain_transaction_timeout(chain));
    let expired = elapsed > timeout;

    match result {
        Ok(update) => Ok(if expired && !update.state.is_completed() {
            TransactionUpdate::new_state(TransactionState::Failed)
        } else {
            update
        }),
        err @ Err(TransactionStatusError::NetworkError(_)) => err,
        Err(_) if expired => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_transaction_update() {
        let chain = Chain::Ethereum;
        let now = || Utc::now().timestamp_millis();
        let pending = || Ok(TransactionUpdate::new_state(TransactionState::Pending));
        let confirmed = || Ok(TransactionUpdate::new_state(TransactionState::Confirmed));

        assert_eq!(
            get_transaction_update(chain, now(), pending()).unwrap().state,
            TransactionState::Pending
        );
        assert_eq!(
            get_transaction_update(chain, 0, pending()).unwrap().state,
            TransactionState::Failed
        );
        assert_eq!(
            get_transaction_update(chain, 0, confirmed()).unwrap().state,
            TransactionState::Confirmed
        );
    }
}
