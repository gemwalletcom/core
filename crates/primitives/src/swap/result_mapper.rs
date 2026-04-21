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

    fn metadata() -> TransactionSwapMetadata {
        TransactionSwapMetadata {
            from_asset: crate::Asset::mock_eth().id,
            from_value: "50000".to_string(),
            to_asset: crate::Asset::mock_eth().id,
            to_value: "2500".to_string(),
            provider: Some("thorchain".to_string()),
        }
    }

    #[test]
    fn test_map_completed_no_metadata() {
        let update = map_swap_result(&SwapResult {
            status: SwapStatus::Completed,
            metadata: None,
        });
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_completed_with_metadata() {
        let meta = metadata();
        let update = map_swap_result(&SwapResult {
            status: SwapStatus::Completed,
            metadata: Some(meta.clone()),
        });
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(matches!(&update.changes[0], TransactionChange::Metadata(TransactionMetadata::Swap(m)) if *m == meta));
    }

    #[test]
    fn test_map_failed() {
        let update = map_swap_result(&SwapResult {
            status: SwapStatus::Failed,
            metadata: None,
        });
        assert_eq!(update.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_in_transit_no_metadata_embed() {
        let update = map_swap_result(&SwapResult {
            status: SwapStatus::InTransit,
            metadata: Some(metadata()),
        });
        assert_eq!(update.state, TransactionState::InTransit);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_pending() {
        let update = map_swap_result(&SwapResult {
            status: SwapStatus::Pending,
            metadata: None,
        });
        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }
}
