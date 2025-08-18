use crate::block_explorer::BlockExplorer;

pub struct TonScan;

impl TonScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(TonScanCustom)
    }
}

// Custom implementation needed because token_url uses "jetton" and validator_url uses address_path
struct TonScanCustom;

impl BlockExplorer for TonScanCustom {
    fn name(&self) -> String {
        "Tonscan".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://tonscan.org/tx/{}", hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://tonscan.org/address/{}", address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("https://tonscan.org/jetton/{}", token))
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        Some(self.get_address_url(validator))
    }
}
