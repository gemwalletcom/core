use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(latest_block: i64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(latest_block as u64);
    let latest_block_number = Some(latest_block as u64);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let latest_block = 12345i64;
        let mapped = map_node_status(latest_block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345));
        assert_eq!(mapped.current_block_number, Some(12345));
    }
}