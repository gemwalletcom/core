use crate::rpc::model::EthSyncingStatus;
use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(sync_status: &EthSyncingStatus) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    match sync_status {
        EthSyncingStatus::NotSyncing(_) => Ok(NodeSyncStatus::new(true, None, None)),
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
        let mapped = map_node_status(&status).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, None);
        assert_eq!(mapped.current_block_number, None);
    }

    #[test]
    fn test_map_node_status_syncing() {
        let info = EthSyncingInfo {
            current_block: BigUint::from(5u64),
            highest_block: BigUint::from(10u64),
        };
        let status = EthSyncingStatus::Syncing(info);

        let mapped = map_node_status(&status).unwrap();

        assert!(!mapped.in_sync);
        assert_eq!(mapped.current_block_number, Some(5));
        assert_eq!(mapped.latest_block_number, Some(10));
    }
}
