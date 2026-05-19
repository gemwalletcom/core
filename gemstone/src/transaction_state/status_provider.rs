use chrono::{DateTime, Utc};
use primitives::{Chain, TransactionChange, TransactionMetadata, TransactionState, TransactionUpdate, chain_transaction_timeout, swap_transaction_timeout};
use std::sync::Arc;
use swapper::{SwapperProvider, swapper::GemSwapper};

use crate::gateway::ChainClientFactory;
use crate::models::{GemTransactionStateRequest, GemTransactionSwapStateRequest};

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
        let result = self.chain_status(chain, request).await;
        get_transaction_update(chain, None, created_at, result)
    }

    pub async fn get_swap_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let created_at = request.transaction.created_at;
        let destination_chain = request.destination_chain;
        let result = self.swap_transaction_status(chain, request).await;
        get_transaction_update(chain, Some(destination_chain), created_at, result)
    }

    async fn swap_transaction_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        if !request.swap_provider.is_cross_chain() {
            return self.chain_status(chain, request.transaction).await;
        }
        self.cross_chain_swap_status(chain, request).await
    }

    async fn cross_chain_swap_status(&self, chain: Chain, request: GemTransactionSwapStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        match request.state {
            TransactionState::Pending => {
                let source_chain_update = self.chain_status(chain, request.transaction).await?;
                Ok(pending_cross_chain_swap_update(source_chain_update))
            }
            TransactionState::InTransit => {
                let swap_update = self.swap_provider_status(chain, request.swap_provider, &request.transaction.id).await?;
                Ok(in_transit_swap_update(swap_update))
            }
            state @ (TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted) => Ok(TransactionUpdate::new_state(state)),
        }
    }

    async fn chain_status(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<TransactionUpdate, TransactionStatusError> {
        let provider = self.chain_factory.create(chain).await?;
        provider
            .get_transaction_status(request.into())
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))
    }

    async fn swap_provider_status(&self, chain: Chain, provider: SwapperProvider, transaction_hash: &str) -> Result<TransactionUpdate, TransactionStatusError> {
        let result = self
            .swapper
            .get_swap_result(chain, provider, transaction_hash)
            .await
            .map_err(|e| TransactionStatusError::NetworkError(e.to_string()))?;

        let state = result.status.transaction_state().unwrap_or(TransactionState::InTransit);
        let changes = result.metadata.map(|m| vec![TransactionChange::Metadata(TransactionMetadata::Swap(m))]).unwrap_or_default();
        Ok(TransactionUpdate::new(state, changes))
    }
}

fn pending_cross_chain_swap_update(source_update: TransactionUpdate) -> TransactionUpdate {
    match source_update.state {
        TransactionState::Confirmed => TransactionUpdate::new(TransactionState::InTransit, source_update.changes),
        _ => source_update,
    }
}

fn in_transit_swap_update(swap_update: TransactionUpdate) -> TransactionUpdate {
    let changes = swap_update.changes;
    TransactionUpdate::new(cross_chain_swap_state(swap_update.state), changes)
}

fn cross_chain_swap_state(swap_state: TransactionState) -> TransactionState {
    match swap_state {
        TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => swap_state,
        TransactionState::Pending | TransactionState::InTransit => TransactionState::InTransit,
    }
}

fn transaction_timeout(chain: Chain, destination_chain: Option<Chain>, state: TransactionState) -> Option<i64> {
    match state {
        TransactionState::Pending => Some(i64::from(chain_transaction_timeout(chain))),
        TransactionState::InTransit => Some(swap_transaction_timeout(chain, destination_chain.unwrap_or(chain)) as i64),
        TransactionState::Confirmed | TransactionState::Failed | TransactionState::Reverted => None,
    }
}

fn get_transaction_update(
    chain: Chain,
    destination_chain: Option<Chain>,
    created_at: DateTime<Utc>,
    result: Result<TransactionUpdate, TransactionStatusError>,
) -> Result<TransactionUpdate, TransactionStatusError> {
    let elapsed = (Utc::now() - created_at).num_milliseconds();
    let pending_expired = elapsed > i64::from(chain_transaction_timeout(chain));

    match result {
        Ok(update) => Ok(if transaction_timeout(chain, destination_chain, update.state).is_some_and(|timeout| elapsed > timeout) {
            TransactionUpdate::new_state(TransactionState::Failed)
        } else {
            update
        }),
        err @ Err(TransactionStatusError::NetworkError(_)) => err,
        Err(_) if pending_expired => Ok(TransactionUpdate::new_state(TransactionState::Failed)),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::TransactionSwapMetadata;

    #[test]
    fn test_get_transaction_update() {
        let chain = Chain::Ethereum;
        let now = Utc::now;
        let pending = || Ok(TransactionUpdate::new_state(TransactionState::Pending));
        let in_transit = || Ok(TransactionUpdate::new_state(TransactionState::InTransit));
        let confirmed = || Ok(TransactionUpdate::new_state(TransactionState::Confirmed));

        assert_eq!(get_transaction_update(chain, None, now(), pending()).unwrap().state, TransactionState::Pending);
        assert_eq!(
            get_transaction_update(chain, None, DateTime::<Utc>::UNIX_EPOCH, pending()).unwrap().state,
            TransactionState::Failed
        );
        assert_eq!(
            get_transaction_update(chain, Some(Chain::Solana), now() - chrono::Duration::hours(3), in_transit())
                .unwrap()
                .state,
            TransactionState::InTransit
        );
        assert_eq!(
            get_transaction_update(chain, Some(chain), now() - chrono::Duration::hours(3), in_transit()).unwrap().state,
            TransactionState::Failed
        );
        assert_eq!(
            get_transaction_update(chain, Some(Chain::Solana), DateTime::<Utc>::UNIX_EPOCH, confirmed()).unwrap().state,
            TransactionState::Confirmed
        );
    }

    #[test]
    fn test_pending_cross_chain_swap_update_moves_confirmed_source_to_in_transit() {
        let source_update = TransactionUpdate::new(
            TransactionState::Confirmed,
            vec![
                TransactionChange::HashChange {
                    old: "broadcast_hash".into(),
                    new: "source_hash".into(),
                },
                TransactionChange::NetworkFee(BigInt::from(123_u32)),
            ],
        );

        let update = pending_cross_chain_swap_update(source_update);

        assert_eq!(update.state, TransactionState::InTransit);
        assert!(matches!(update.changes.first(), Some(TransactionChange::HashChange { old, new }) if old == "broadcast_hash" && new == "source_hash"));
        assert!(update.changes.iter().any(|change| matches!(change, TransactionChange::NetworkFee(_))));
    }

    #[test]
    fn test_pending_cross_chain_swap_update_keeps_non_confirmed_source_state() {
        let source_update = TransactionUpdate::new_state(TransactionState::Pending);

        let update = pending_cross_chain_swap_update(source_update);

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_in_transit_swap_update_keeps_final_swap_metadata() {
        let metadata = TransactionSwapMetadata {
            from_asset: Chain::Ton.as_asset_id(),
            from_value: "1000000".into(),
            to_asset: Chain::Solana.as_asset_id(),
            to_value: "966847".into(),
            provider: Some("near_intents".into()),
        };
        let swap_update = TransactionUpdate::new(TransactionState::Confirmed, vec![TransactionChange::Metadata(TransactionMetadata::Swap(metadata.clone()))]);

        let update = in_transit_swap_update(swap_update);

        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(
            update
                .changes
                .iter()
                .any(|change| matches!(change, TransactionChange::Metadata(TransactionMetadata::Swap(value)) if value == &metadata))
        );
    }

    #[test]
    fn test_cross_chain_swap_state_maps_non_terminal_to_in_transit() {
        assert_eq!(cross_chain_swap_state(TransactionState::Pending), TransactionState::InTransit);
        assert_eq!(cross_chain_swap_state(TransactionState::InTransit), TransactionState::InTransit);
        assert_eq!(cross_chain_swap_state(TransactionState::Confirmed), TransactionState::Confirmed);
        assert_eq!(cross_chain_swap_state(TransactionState::Failed), TransactionState::Failed);
        assert_eq!(cross_chain_swap_state(TransactionState::Reverted), TransactionState::Reverted);
    }
}
