use crate::block_explorer::{BlockExplorer, Metadata};

pub struct HyperLiquid {
    pub meta: Metadata,
}

impl HyperLiquid {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Hyper Explorer",
                base_url: "https://hyperevm-explorer.vercel.app",
            },
        })
    }
}

impl BlockExplorer for HyperLiquid {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.get_address_url(validator).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tx_url() {
        let explorer = HyperLiquid::new();
        let address = "0x0361d1ce0520968024f140cdedaa3c21f793c5a7";
        let tx_id = "0x971f5a256584fae7248b5eefa5d2e85b5522192aed198a61ae6b542129822659";

        assert_eq!(
            explorer.get_tx_url(tx_id),
            "https://hyperevm-explorer.vercel.app/tx/0x971f5a256584fae7248b5eefa5d2e85b5522192aed198a61ae6b542129822659"
        );
        assert_eq!(
            explorer.get_address_url(address),
            "https://hyperevm-explorer.vercel.app/address/0x0361d1ce0520968024f140cdedaa3c21f793c5a7"
        );
    }
}
