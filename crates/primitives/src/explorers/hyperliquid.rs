use crate::block_explorer::{BlockExplorer, Metadata};

pub struct HyperliquidExplorer {
    pub meta: Metadata,
}

impl HyperliquidExplorer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Hyperliquid Explorer",
                base_url: "https://app.hyperliquid.xyz/explorer",
            },
        })
    }
}

impl BlockExplorer for HyperliquidExplorer {
    fn name(&self) -> String {
        self.meta.name.to_string()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}/token/{}", self.meta.base_url, token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperliquid_explorer_name() {
        let explorer = HyperliquidExplorer::new();
        assert_eq!(explorer.name(), "Hyperliquid Explorer");
    }

    #[test]
    fn test_hyperliquid_explorer_tx_url() {
        let explorer = HyperliquidExplorer::new();
        let tx_hash = "0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86";
        let result = explorer.get_tx_url(tx_hash);
        assert_eq!(result, "https://app.hyperliquid.xyz/explorer/tx/0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86");
    }

    #[test]
    fn test_hyperliquid_explorer_address_url() {
        let explorer = HyperliquidExplorer::new();
        let address = "0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce";
        let result = explorer.get_address_url(address);
        assert_eq!(result, "https://app.hyperliquid.xyz/explorer/address/0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce");
    }

    #[test]
    fn test_hyperliquid_explorer_token_url() {
        let explorer = HyperliquidExplorer::new();
        let token = "0x1234567890abcdef1234567890abcdef12345678";
        let result = explorer.get_token_url(token);
        assert_eq!(result, Some("https://app.hyperliquid.xyz/explorer/token/0x1234567890abcdef1234567890abcdef12345678".to_string()));
    }
}
