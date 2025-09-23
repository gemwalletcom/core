use crate::models::Chainhead;
use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(chainhead: &Chainhead) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    let current_block = Some(chainhead.last.seqno);
    let latest_block_number = Some(chainhead.last.seqno);
    Ok(NodeSyncStatus::new(true, latest_block_number, current_block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BlockInfo, Chainhead};

    #[test]
    fn test_map_node_status() {
        let block_info = BlockInfo {
            seqno: 12345,
            root_hash: String::new(),
        };
        let chainhead = Chainhead {
            first: block_info.clone(),
            last: block_info,
        };
        let mapped = map_node_status(&chainhead).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345));
        assert_eq!(mapped.current_block_number, Some(12345));
    }
}
