use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub fn new() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata::new("Mempool", "https://mempool.space"))
}
