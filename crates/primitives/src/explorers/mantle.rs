use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct MantleExplorer;

impl MantleExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("Mantle Explorer", "https://explorer.mantle.xyz"))
    }
}
