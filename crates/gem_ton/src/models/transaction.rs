use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

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
pub struct MessageTransactions {
    pub transactions: Vec<TransactionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub hash: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub total_fees: BigUint,
    pub description: Option<TransactionDescription>,
    pub out_msgs: Vec<OutMessage>,
    pub in_msg: Option<TransactionInMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub source: String,
    pub destination: Option<String>,
    pub value: Option<String>,
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
pub struct InMessage {
    pub hash: String,
    pub msg_type: Option<String>,
    pub value: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
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
pub struct TransactionInMessage {
    pub hash: String,
    pub source: Option<String>,
    pub destination: String,
    pub value: Option<String>,
    pub opcode: Option<String>,
    pub bounce: Option<bool>,
    pub bounced: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDescription {
    pub aborted: bool,
    pub compute_ph: Option<ComputePhase>,
    pub action: Option<ActionPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePhase {
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPhase {
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastTransaction {
    pub hash: String,
}
