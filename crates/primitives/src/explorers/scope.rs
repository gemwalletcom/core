use crate::block_explorer::{BlockExplorer, Metadata};
use crate::Chain;

pub struct ScopeExplorer {
    pub meta: Metadata,
    pub chain_id: String,
}

impl ScopeExplorer {
    pub fn new(chain: Chain) -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Scope.sh",
                base_url: "https://scope.sh",
            },
            chain_id: chain.network_id().to_string(),
        })
    }
}

impl BlockExplorer for ScopeExplorer {
    fn name(&self) -> String {
        self.meta.name.into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/tx/{}", self.meta.base_url, self.chain_id, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}/address/{}", self.meta.base_url, self.chain_id, address).to_string()
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(self.get_address_url(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_explorer_tx_url() {
        let explorer = ScopeExplorer::new(Chain::Ethereum);
        let tx_hash = "0x72d1ca34265d6d600df0f9ef1a43eb2995a7c2b784f99306ced2b917f0757d55";

        assert_eq!(
            explorer.get_tx_url(tx_hash),
            "https://scope.sh/1/tx/0x72d1ca34265d6d600df0f9ef1a43eb2995a7c2b784f99306ced2b917f0757d55"
        );
    }

    #[test]
    fn test_scope_explorer_address_url() {
        let explorer = ScopeExplorer::new(Chain::Ethereum);
        let address = "0xc9f5296eb3ac266c94568d790b6e91eba7d76a11";

        assert_eq!(
            explorer.get_address_url(address),
            "https://scope.sh/1/address/0xc9f5296eb3ac266c94568d790b6e91eba7d76a11"
        );
    }

    #[test]
    fn test_scope_explorer_token_url() {
        let explorer = ScopeExplorer::new(Chain::Ethereum);
        let token_address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";

        assert_eq!(
            explorer.get_token_url(token_address),
            Some("https://scope.sh/1/address/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string())
        );

        // Test with a different chain_id
        let explorer_polygon = ScopeExplorer::new(Chain::Polygon);
        assert_eq!(
            explorer_polygon.get_token_url(token_address),
            Some("https://scope.sh/137/address/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string())
        );
    }
}
