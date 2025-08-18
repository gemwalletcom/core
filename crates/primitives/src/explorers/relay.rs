use crate::block_explorer::BlockExplorer;

pub struct RelayScan;

impl RelayScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(RelayExplorer)
    }
}

// Custom implementation needed for query parameter pattern
struct RelayExplorer;

impl BlockExplorer for RelayExplorer {
    fn name(&self) -> String {
        "Relay".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://relay.link/transaction/{}", hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://relay.link/transaction?address={}", address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_scan() {
        let relay_scan = RelayScan::boxed();
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
