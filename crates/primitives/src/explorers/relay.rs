use crate::block_explorer::{BlockExplorer, Metadata};

pub struct RelayScan {
    pub meta: Metadata,
}

impl RelayScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Relay",
                base_url: "https://relay.link/transaction",
            },
        })
    }
}

impl BlockExplorer for RelayScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}?address={}", self.meta.base_url, address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_scan() {
        let relay_scan = RelayScan::new();
        let address = "0x4dece432bd65b664b9f92b983231dac48eccfa19";
        let tx = "0x1d2a1cc47871b3779457dacd61db6e122ded1d5875e0c71650337386ef95d9b4";

        assert_eq!(relay_scan.name(), "Relay");
        assert_eq!(
            relay_scan.get_tx_url(tx),
            "https://relay.link/transaction/0x1d2a1cc47871b3779457dacd61db6e122ded1d5875e0c71650337386ef95d9b4"
        );
        assert_eq!(
            relay_scan.get_address_url(address),
            "https://relay.link/transaction?address=0x4dece432bd65b664b9f92b983231dac48eccfa19"
        );
    }
}
