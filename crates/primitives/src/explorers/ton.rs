use crate::block_explorer::BlockExplorer;

pub fn new_ton_viewer() -> Box<dyn BlockExplorer> {
    Box::new(TonViewer)
}

pub struct TonViewer;

impl BlockExplorer for TonViewer {
    fn name(&self) -> String {
        "TonViewer".into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://tonviewer.com/transaction/{}", hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("https://tonviewer.com/{}", address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("https://tonviewer.com/{}", token))
    }
}