use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(latest_checkpoint: u64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(latest_checkpoint);
    let latest_block_number = Some(latest_checkpoint);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let latest_checkpoint = 98765u64;
        let mapped = map_node_status(latest_checkpoint).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(98765));
        assert_eq!(mapped.current_block_number, Some(98765));
    }
}
