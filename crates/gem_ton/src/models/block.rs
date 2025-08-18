use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonMasterchainInfo {
    pub last: TonBlock,
    //init: TonBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonBlock {
    pub seqno: i32,
    pub root_hash: String,
}
