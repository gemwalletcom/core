use chrono::Utc;
use gem_jsonrpc::alien::RpcProvider;
use primitives::swap::map_swap_result;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate, chain_transaction_timeout};
use std::sync::Arc;
use swapper::swapper::GemSwapper;

use crate::gateway::ChainClientFactory;

use super::{ResolverError, TransactionStateInput};

pub struct TransactionStatusResolver {
    chain_factory: Arc<ChainClientFactory>,
    swapper: Arc<GemSwapper>,
}

impl TransactionStatusResolver {
    pub fn new(chain_factory: Arc<ChainClientFactory>, rpc: Arc<dyn RpcProvider>) -> Self {
        Self {
            chain_factory,
            swapper: Arc::new(GemSwapper::new(rpc)),
        }
    }

    pub async fn status(&self, input: TransactionStateInput) -> Result<TransactionUpdate, ResolverError> {
        let elapsed = Utc::now().timestamp().saturating_sub(input.created_at_secs).max(0) as u64;
        let timeout = chain_transaction_timeout(input.chain) as u64 / 1000;

        match input.state {
            TransactionState::Pending => {
                let provider = self.chain_factory.create(input.chain).await?;
                match provider.get_transaction_status(TransactionStateRequest::from(&input)).await {
                    Ok(update) => Ok(apply_timeout(update, elapsed, timeout)),
                    Err(_) if elapsed > timeout => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
                    Err(err) => Err(ResolverError::NetworkError(err.to_string())),
                }
            }
            TransactionState::InTransit => {
                let Some(provider) = input.swap_provider() else {
                    return Ok(TransactionUpdate::new_state(TransactionState::Failed));
                };
                let result = self.swapper.get_swap_result(input.chain, provider, &input.hash).await?;
                Ok(apply_timeout(map_swap_result(&result), elapsed, timeout))
            }
            TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => Ok(TransactionUpdate::new_state(input.state)),
        }
    }
}

fn apply_timeout(update: TransactionUpdate, elapsed_secs: u64, timeout_secs: u64) -> TransactionUpdate {
    if !update.state.is_terminal() && elapsed_secs > timeout_secs {
        TransactionUpdate::new_state(TransactionState::Failed)
    } else {
        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_timeout() {
        assert_eq!(apply_timeout(TransactionUpdate::new_state(TransactionState::Pending), 50, 60).state, TransactionState::Pending);
        assert_eq!(apply_timeout(TransactionUpdate::new_state(TransactionState::Pending), 61, 60).state, TransactionState::Failed);
        assert_eq!(apply_timeout(TransactionUpdate::new_state(TransactionState::InTransit), 61, 60).state, TransactionState::Failed);
        assert_eq!(apply_timeout(TransactionUpdate::new_state(TransactionState::Confirmed), 120, 60).state, TransactionState::Confirmed);
        assert_eq!(apply_timeout(TransactionUpdate::new_state(TransactionState::Failed), 120, 60).state, TransactionState::Failed);
    }
}
