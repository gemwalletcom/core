use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, ADDRESS_PATH, TRANSACTIONS_PATH, TXNS_PATH};

pub struct NearBlocks;

impl NearBlocks {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Near",
            base_url: "https://nearblocks.io",
            tx_path: TXNS_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}

pub struct NearIntentsExplorer;

impl NearIntentsExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "NEAR Intents",
            base_url: "https://explorer.near-intents.org",
            tx_path: TRANSACTIONS_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_blocks_tx_url() {
        let explorer = NearIntentsExplorer::boxed();
        let deposit_address = "0x2D097B18f80f60861228B7F76dC0F525D89dBE6e";

        assert_eq!(
            explorer.get_tx_url(deposit_address),
            "https://explorer.near-intents.org/transactions/0x2D097B18f80f60861228B7F76dC0F525D89dBE6e"
        );
    }
}
