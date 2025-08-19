use crate::block_explorer::BlockExplorer;

pub struct XrpScan;

impl XrpScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(XrpScanExplorer)
    }
}

// Custom implementation needed because token_url uses address_path
struct XrpScanExplorer;

impl BlockExplorer for XrpScanExplorer {
    fn name(&self) -> String {
        "XrpScan".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://xrpscan.com/tx/{}", hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://xrpscan.com/account/{}", address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(self.get_address_url(token))
    }
}
