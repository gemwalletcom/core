use crate::block_explorer::{BlockExplorer, Metadata};

pub struct ChainflipScan {
    pub meta: Metadata,
}

impl ChainflipScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Chainflip",
                base_url: "https://scan.chainflip.io",
            },
        })
    }
}

impl BlockExplorer for ChainflipScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, _address: &str) -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chainflip_scan() {
        let chainflip_scan = ChainflipScan::new();
        let tx = "54qsbkVUPoQUbwfuQeDXmNyodPWVX8VcK6sSSFyfezkg8t5XbduthFisKBcGxGjSab8QsKaPoEWEnzsK9xsFXrMF";
        assert_eq!(chainflip_scan.name(), "Chainflip");
        assert_eq!(
            chainflip_scan.get_tx_url(tx),
            "https://scan.chainflip.io/tx/54qsbkVUPoQUbwfuQeDXmNyodPWVX8VcK6sSSFyfezkg8t5XbduthFisKBcGxGjSab8QsKaPoEWEnzsK9xsFXrMF"
        );
    }
}
