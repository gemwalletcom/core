use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chainhead {
    pub last: BlockInfo,
    #[serde(rename = "first")]
    pub first: BlockInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub seqno: u64,
    pub root_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRef {
    pub workchain: i32,
    pub shard: String,
    pub seqno: i64,
}
