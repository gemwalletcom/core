use super::near::NEAR_BLOCKS_BASE_URL;
use crate::block_explorer::BlockExplorer;

pub struct NearIntents;

impl NearIntents {
    const BASE_URL: &'static str = "https://explorer.near-intents.org";

    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(Self)
    }
}

impl BlockExplorer for NearIntents {
    fn name(&self) -> String {
        "NEAR Intents".to_string()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transactions/{}", Self::BASE_URL, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{NEAR_BLOCKS_BASE_URL}/address/{address}")
    }
}
