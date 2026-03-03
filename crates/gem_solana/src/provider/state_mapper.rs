use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(slot: u64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    Ok(NodeSyncStatus::synced(slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let slot = 287654321u64;
        let mapped = map_node_status(slot).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(287654321));
        assert_eq!(mapped.current_block_number, Some(287654321));
    }
}
