use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct Viewblock;

impl Viewblock {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::new("Viewblock", "https://viewblock.io/thorchain"))
    }
}

pub struct RuneScan;

impl RuneScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(RuneScanExplorer)
    }
}

// Custom implementation needed for hash trimming
struct RuneScanExplorer;

impl BlockExplorer for RuneScanExplorer {
    fn name(&self) -> String {
        "RuneScan".into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://runescan.io/tx/{}", hash.trim_start_matches("0x"))
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("https://runescan.io/address/{}", address)
    }
}
