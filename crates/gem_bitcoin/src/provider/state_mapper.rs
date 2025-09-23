use crate::models::block::BitcoinNodeInfo;
use primitives::NodeSyncStatus;

pub fn map_node_status(node_info: &BitcoinNodeInfo) -> NodeSyncStatus {
    let latest_block_number = node_info.backend.as_ref().map(|backend| backend.blocks);
    let current_block_number = Some(node_info.blockbook.best_height);

    NodeSyncStatus::new(node_info.blockbook.in_sync, latest_block_number, current_block_number)
}

pub fn map_latest_block_number(node_info: &BitcoinNodeInfo) -> u64 {
    node_info
        .backend
        .as_ref()
        .map(|backend| backend.blocks)
        .unwrap_or(node_info.blockbook.best_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::block::{BitcoinBackend, BitcoinBlockbook, BitcoinNodeInfo};

    #[test]
    fn test_map_node_status_returns_flag_and_block_numbers() {
        let node_info = BitcoinNodeInfo {
            blockbook: BitcoinBlockbook {
                in_sync: false,
                last_block_time: "2024-01-01T00:00:00Z".to_string(),
                best_height: 123,
            },
            backend: Some(BitcoinBackend {
                blocks: 456,
                chain: Some("main".to_string()),
            }),
        };

        let status = map_node_status(&node_info);

        assert!(!status.in_sync);
        assert_eq!(status.latest_block_number, Some(456));
        assert_eq!(status.current_block_number, Some(123));
    }

    #[test]
    fn test_map_latest_block_number_returns_best_height() {
        let node_info = BitcoinNodeInfo {
            blockbook: BitcoinBlockbook {
                in_sync: true,
                last_block_time: "2024-01-01T00:00:00Z".to_string(),
                best_height: 1_000,
            },
            backend: Some(BitcoinBackend {
                blocks: 2_000,
                chain: Some("main".to_string()),
            }),
        };

        assert_eq!(map_latest_block_number(&node_info), 2_000);
    }
}
