use serde::{Serialize, Deserialize};

// node

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResultBlockType {
    pub last: NodeResultBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResultBlock {
    pub seqno: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResultShardsType {
    pub shards: Vec<NodeResultBlock>,
}

// api
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainHead {
    pub seqno: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub transactions: Vec<Transaction>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub total_fees: i64,
    pub success: bool,
    pub transaction_type: String,
    pub in_msg: Option<InMessage>,
    pub out_msgs: Vec<OutMessage>,
    pub block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMessage {
    pub bounced: bool,
    pub destination: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub value: i64,
    pub bounced: bool,
    pub source: Address,
    pub destination: Option<Address>,
    pub decoded_op_name: Option<String>,
    pub decoded_body: Option<DecodedBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedBody {
    pub text: Option<String>,
}