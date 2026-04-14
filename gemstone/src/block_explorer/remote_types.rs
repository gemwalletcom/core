use primitives::block_explorer::ExplorerInput;

pub type GemExplorerInput = ExplorerInput;

#[uniffi::remote(Record)]
pub struct GemExplorerInput {
    pub tx_hash: String,
    pub recipient: Option<String>,
    pub memo: Option<String>,
}
