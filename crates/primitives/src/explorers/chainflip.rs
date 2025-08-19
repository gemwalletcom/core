use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH};

pub struct ChainflipScan;

impl ChainflipScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Chainflip",
            base_url: "https://scan.chainflip.io",
            tx_path: TX_PATH,
            address_path: "",
            token_path: None,
            validator_path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chainflip_scan() {
        let chainflip_scan = ChainflipScan::boxed();
        let tx = "54qsbkVUPoQUbwfuQeDXmNyodPWVX8VcK6sSSFyfezkg8t5XbduthFisKBcGxGjSab8QsKaPoEWEnzsK9xsFXrMF";
        assert_eq!(chainflip_scan.name(), "Chainflip");
        assert_eq!(
            chainflip_scan.get_tx_url(tx),
            "https://scan.chainflip.io/tx/54qsbkVUPoQUbwfuQeDXmNyodPWVX8VcK6sSSFyfezkg8t5XbduthFisKBcGxGjSab8QsKaPoEWEnzsK9xsFXrMF"
        );
    }
}
