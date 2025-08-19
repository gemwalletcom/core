use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct SocketScan;

impl SocketScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::new("SocketScan", "https://socketscan.io"))
    }
}
