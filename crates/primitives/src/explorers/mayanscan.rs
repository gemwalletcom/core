use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct MayanScan;

impl MayanScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::new("Mayan Explorer", "https://explorer.mayan.finance"))
    }
}
