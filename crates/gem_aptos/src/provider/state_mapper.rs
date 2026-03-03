use crate::models::Ledger;
use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(ledger: &Ledger) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    Ok(NodeSyncStatus::synced(ledger.block_height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let ledger = Ledger {
            chain_id: 1,
            block_height: 987654321,
            epoch: 13156,
            ledger_timestamp: 1759855099031447,
        };
        let mapped = map_node_status(&ledger).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(987654321));
        assert_eq!(mapped.current_block_number, Some(987654321));
    }
}
