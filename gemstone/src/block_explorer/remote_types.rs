use primitives::block_explorer::SwapExplorerInput;

pub type GemSwapExplorerInput = SwapExplorerInput;

#[uniffi::remote(Record)]
pub struct GemSwapExplorerInput {
    pub tx_hash: String,
    pub recipient: Option<String>,
    pub memo: Option<String>,
}
