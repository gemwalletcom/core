use crate::rpc::model::EthSyncingStatus;
use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(sync_status: &EthSyncingStatus, latest_block: u64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    match sync_status {
        EthSyncingStatus::NotSyncing(_) => Ok(NodeSyncStatus::new(true, Some(latest_block), Some(latest_block))),
        EthSyncingStatus::Syncing(info) => {
            let latest = info.highest_block.to_string().parse::<u64>().ok();
            let current = info.current_block.to_string().parse::<u64>().ok();
            Ok(NodeSyncStatus::new(false, latest, current))
        }
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use crate::rpc::model::EthSyncingInfo;

    use super::*;

    #[test]
    fn test_map_node_status_not_syncing() {
        let status = EthSyncingStatus::NotSyncing(false);
        let latest_block = 12345678u64;
        let mapped = map_node_status(&status, latest_block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345678));
        assert_eq!(mapped.current_block_number, Some(12345678));
    }

    #[test]
    fn test_map_node_status_syncing() {
        let info = EthSyncingInfo {
            current_block: BigUint::from(5u64),
            highest_block: BigUint::from(10u64),
        };
        let status = EthSyncingStatus::Syncing(info);
        let latest_block = 12345678u64;

        let mapped = map_node_status(&status, latest_block).unwrap();

        assert!(!mapped.in_sync);
        assert_eq!(mapped.current_block_number, Some(5));
        assert_eq!(mapped.latest_block_number, Some(10));
    }
}
