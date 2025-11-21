use crate::block_explorer::BlockExplorer;

const OKX_BASE_URL: &str = "https://www.okx.com/web3/explorer";

pub struct OkxExplorer;

impl OkxExplorer {
    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Box::new(OkxChainExplorer { chain: "inkchain" })
    }

    pub fn new_xlayer() -> Box<dyn BlockExplorer> {
        Box::new(OkxChainExplorer { chain: "xlayer" })
    }
}

struct OkxChainExplorer {
    chain: &'static str,
}

impl BlockExplorer for OkxChainExplorer {
    fn name(&self) -> String {
        "OKX Explorer".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/{}/tx/{}", OKX_BASE_URL, self.chain, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/{}/address/{}", OKX_BASE_URL, self.chain, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}/{}/token/{}", OKX_BASE_URL, self.chain, token))
    }
}

#[cfg(test)]
mod tests {
    use super::OkxExplorer;

    #[test]
    fn test_ink_tx_url() {
        let explorer = OkxExplorer::new_ink();
        let tx_id = "0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5";

        assert_eq!(
            explorer.get_tx_url(tx_id),
            "https://www.okx.com/web3/explorer/inkchain/tx/0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5"
        )
    }

    #[test]
    fn test_xlayer_tx_url() {
        let explorer = OkxExplorer::new_xlayer();
        let tx_id = "0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5";

        assert_eq!(
            explorer.get_tx_url(tx_id),
            "https://www.okx.com/web3/explorer/xlayer/tx/0x37a2d85b95d881be32fb806a5c50bfac320565019e408cc6e3aa2072a8929cf5"
        )
    }
}
