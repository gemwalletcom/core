use crate::block_explorer::BlockExplorer;

pub struct ZkSync;

impl ZkSync {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(ZkSyncCustom)
    }
}

// Custom implementation needed because token_url uses address_path
struct ZkSyncCustom;

impl BlockExplorer for ZkSyncCustom {
    fn name(&self) -> String {
        "zkSync.io".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://explorer.zksync.io/tx/{}", hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://explorer.zksync.io/address/{}", address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(self.get_address_url(token))
    }
}
