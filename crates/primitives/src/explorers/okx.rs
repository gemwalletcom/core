use crate::block_explorer::{BlockExplorer, Metadata};

pub struct OkxExplorer {
    pub meta: Metadata,
    pub chain_path: &'static str,
}

impl OkxExplorer {
    pub fn new_ink() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "OKX Explorer",
                base_url: "https://www.okx.com/web3/explorer",
            },
            chain_path: "inkchain",
        })
    }
}

impl BlockExplorer for OkxExplorer {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/tx/{}", self.meta.base_url, self.chain_path, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}/address/{}", self.meta.base_url, self.chain_path, address)
    }
    fn get_token_url(&self, _token: &str) -> Option<String> {
        Some(format!("{}/{}/token/{}", self.meta.base_url, self.chain_path, _token))
    }
}

#[cfg(test)]
mod tests {
    use crate::block_explorer::BlockExplorer;

    use super::OkxExplorer;

    #[test]
    fn test_tx_url() {
        let explorer = OkxExplorer::new_ink();
        let tx_id = "0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5";

        assert_eq!(
            explorer.get_tx_url(tx_id),
            "https://www.okx.com/web3/explorer/inkchain/tx/0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5"
        )
    }
}
