use chrono::Utc;
use primitives::{Chain, TransactionChange, TransactionMetadata, TransactionState, TransactionStateRequest, TransactionUpdate, chain_transaction_timeout};
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
        let result = match request.swap_provider {
            Some(provider) => self.get_swap_status(chain, provider, &request.id).await,
            None => self.get_chain_status(chain, &request).await,
        };
        get_resolved_status(chain, request.created_at, result)
    }

    async fn get_chain_status(&self, chain: Chain, request: &GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let provider = self.chain_factory.create(chain).await?;
        provider
            .get_transaction_status(TransactionStateRequest {
                id: request.id.clone(),
                sender_address: request.sender_address.clone(),
                created_at: request.created_at,
                block_number: request.block_number,
            })
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

fn get_resolved_status(
    chain: Chain,
    created_at: i64,
    result: Result<TransactionUpdate, TransactionStatusError>,
) -> Result<TransactionUpdate, TransactionStatusError> {
    let elapsed = Utc::now().timestamp().saturating_sub(created_at).max(0) as u64;
    let timeout = chain_transaction_timeout(chain) as u64 / 1000;
    let expired = elapsed > timeout;

    match result {
        Ok(update) => {
            let running = match update.state {
                TransactionState::Pending | TransactionState::InTransit => true,
                TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => false,
            };
            Ok(if expired && running {
                TransactionUpdate::new_state(TransactionState::Failed)
            } else {
                update
            })
        }
        err @ Err(TransactionStatusError::NetworkError(_)) => err,
        Err(_) if expired => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
        Err(err) => Err(err),
    }
}
