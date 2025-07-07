use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonMasterchainInfo {
    pub last: TonBlock,
    //init: TonBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonBlock {
    pub seqno: i32,
    pub root_hash: String,
}
