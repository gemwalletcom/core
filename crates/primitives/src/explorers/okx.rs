use crate::block_explorer::BlockExplorer;

pub struct OkxExplorer;

impl OkxExplorer {
    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Box::new(OkxInkExplorer)
    }
}

// Custom implementation needed for chain_path pattern
struct OkxInkExplorer;

impl BlockExplorer for OkxInkExplorer {
    fn name(&self) -> String {
        "OKX Explorer".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://www.okx.com/web3/explorer/inkchain/tx/{}", hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://www.okx.com/web3/explorer/inkchain/address/{}", address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("https://www.okx.com/web3/explorer/inkchain/token/{}", token))
    }
}

#[cfg(test)]
mod tests {

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
