use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct Cardanocan;

impl Cardanocan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::blockchair("CardanoScan", "https://cardanoscan.io"))
    }
}
