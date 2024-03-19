use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chainhead {
    pub seqno: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shards {
    pub shards: Vec<Shard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocks {
    pub blocks: Vec<Block>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub seqno: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub last_known_block_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub block: String,
    pub transaction_type: String,
    pub total_fees: i64,
    pub out_msgs: Vec<OutMessage>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub source: Address,
    pub destination: Option<Address>,
    pub value: i64,
    pub op_code: Option<String>,
    pub decoded_op_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub address: Address,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPrice {
    pub value: String,
    pub token_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPreview {
    pub resolution: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nft {
    pub index: u8,
    pub owner: Account,
    pub collection: Collection,
    pub verified: bool,
    pub price: NftPrice,
    pub dns: String,
    pub previews: Vec<NftPreview>,
    pub approved_by: Vec<String>,
}
