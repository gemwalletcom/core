use crate::models::rpc::LedgerCurrent;
use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(ledger_info: &LedgerCurrent) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(ledger_info.ledger_current_index as u64);
    let latest_block_number = Some(ledger_info.ledger_current_index as u64);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let ledger_info = LedgerCurrent {
            ledger_current_index: 80123456,
        };
        let mapped = map_node_status(&ledger_info).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(80123456));
        assert_eq!(mapped.current_block_number, Some(80123456));
    }
}