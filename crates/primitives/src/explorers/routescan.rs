use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct RouteScan;

impl RouteScan {
    pub fn new_avax() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("SnowTrace", "https://snowtrace.io"))
    }

    pub fn new_sonic() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("RouteScan", "https://146.routescan.io"))
    }

    pub fn new_ink() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::with_token("RouteScan", "https://57073.routescan.io"))
    }
}
