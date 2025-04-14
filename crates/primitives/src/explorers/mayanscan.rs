use crate::block_explorer::{BlockExplorer, Metadata};

pub struct MayanScan {
    pub meta: Metadata,
}

impl MayanScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "Mayan Explorer",
                base_url: "https://explorer.mayan.finance",
            },
        })
    }
}
impl BlockExplorer for MayanScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    // this method is not supported on mayan
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.meta.base_url, address)
    }
}
