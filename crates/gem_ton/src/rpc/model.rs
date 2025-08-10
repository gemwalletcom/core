use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

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
    pub in_msg: Option<InMessage>,
    pub block: String,
    pub transaction_type: String,
    pub total_fees: i64,
    pub out_msgs: Vec<OutMessage>,
    pub success: bool,
    pub utime: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMessage {
    pub hash: String,
    pub msg_type: Option<String>,
    pub value: Option<i64>,
    pub source: Option<Address>,
    pub destination: Option<Address>,
    pub body: Option<String>,
    pub comment: Option<String>,
    pub decoded_body: Option<DecodedBody>,
}

impl HasMemo for InMessage {
    fn comment(&self) -> &Option<String> {
        &self.comment
    }

    fn decoded_body(&self) -> &Option<DecodedBody> {
        &self.decoded_body
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub source: Address,
    pub destination: Option<Address>,
    pub value: i64,
    pub op_code: Option<String>,
    pub decoded_op_name: Option<String>,
    pub body: Option<String>,
    pub comment: Option<String>,
    pub decoded_body: Option<DecodedBody>,
}

impl HasMemo for OutMessage {
    fn comment(&self) -> &Option<String> {
        &self.comment
    }

    fn decoded_body(&self) -> &Option<DecodedBody> {
        &self.decoded_body
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonInfo {
    pub metadata: JettonInfoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonInfoMetadata {
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub decimals: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonBalances {
    pub balances: Vec<JettonBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonBalance {
    pub balance: String,
    pub jetton: Jetton,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jetton {
    pub address: String,
}

pub trait HasMemo {
    fn comment(&self) -> &Option<String>;
    fn decoded_body(&self) -> &Option<DecodedBody>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedBody {
    #[serde(rename = "type")]
    pub body_type: Option<String>,
    pub comment: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}
