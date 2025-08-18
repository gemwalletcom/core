use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ADDRESS_PATH, TOKEN_PATH};

pub struct HyperliquidExplorer;

impl HyperliquidExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Hyperliquid",
            base_url: "https://app.hyperliquid.xyz/explorer",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperliquid_explorer_tx_url() {
        let explorer = HyperliquidExplorer::boxed();
        let tx_hash = "0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86";
        let result = explorer.get_tx_url(tx_hash);

        assert_eq!(
            result,
            "https://app.hyperliquid.xyz/explorer/tx/0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86"
        );
    }

    #[test]
    fn test_hyperliquid_explorer_address_url() {
        let explorer = HyperliquidExplorer::boxed();
        let address = "0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce";
        let result = explorer.get_address_url(address);

        assert_eq!(
            result,
            "https://app.hyperliquid.xyz/explorer/address/0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce"
        );
    }

    #[test]
    fn test_hyperliquid_explorer_token_url() {
        let explorer = HyperliquidExplorer::boxed();
        let token = "0x0d01dc56dcaaca66ad901c959b4011ec";
        let result = explorer.get_token_url(token).unwrap();

        assert_eq!(result, "https://app.hyperliquid.xyz/explorer/token/0x0d01dc56dcaaca66ad901c959b4011ec");
    }
}
