use super::SwapResult;
use crate::transaction_update::{TransactionChange, TransactionMetadata, TransactionUpdate};

pub fn map_swap_result(result: &SwapResult) -> TransactionUpdate {
    let state = result.status.transaction_state();
    let changes = if state.is_terminal() {
        result
            .metadata
            .clone()
            .map(|m| vec![TransactionChange::Metadata(TransactionMetadata::Swap(m))])
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    TransactionUpdate::new(state, changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionSwapMetadata;
    use crate::swap::SwapStatus;
    use crate::transaction_state::TransactionState;

    #[test]
    fn test_map_swap_result() {
        let meta = TransactionSwapMetadata::mock();

        let completed = map_swap_result(&SwapResult {
            status: SwapStatus::Completed,
            metadata: None,
        });
        assert_eq!(completed.state, TransactionState::Confirmed);
        assert!(completed.changes.is_empty());

        let completed_with_meta = map_swap_result(&SwapResult {
            status: SwapStatus::Completed,
            metadata: Some(meta.clone()),
        });
        assert_eq!(completed_with_meta.state, TransactionState::Confirmed);
        assert!(matches!(&completed_with_meta.changes[0], TransactionChange::Metadata(TransactionMetadata::Swap(m)) if *m == meta));

        let failed = map_swap_result(&SwapResult {
            status: SwapStatus::Failed,
            metadata: None,
        });
        assert_eq!(failed.state, TransactionState::Failed);

        let in_transit = map_swap_result(&SwapResult {
            status: SwapStatus::InTransit,
            metadata: Some(meta.clone()),
        });
        assert_eq!(in_transit.state, TransactionState::InTransit);
        assert!(in_transit.changes.is_empty());

        let pending = map_swap_result(&SwapResult {
            status: SwapStatus::Pending,
            metadata: None,
        });
        assert_eq!(pending.state, TransactionState::Pending);
        assert!(pending.changes.is_empty());
    }
}
