use std::collections::HashMap;

use num_bigint::BigUint;
use primitives::TransactionState;
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
pub struct TraceResponse {
    pub traces: Vec<Trace>,
}

impl TraceResponse {
    pub fn root_transaction(&self) -> Option<&TransactionMessage> {
        let trace = self.traces.first()?;
        let transaction_id = trace.transactions_order.first()?;
        trace.transactions.get(transaction_id)
    }

    pub fn action_state(&self) -> Option<TransactionState> {
        self.traces.first().map(Trace::action_state)
    }

    pub fn has_actions(&self) -> bool {
        match self.traces.first() {
            Some(trace) => !trace.actions.is_empty(),
            None => false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TraceByMessageQuery {
    pub msg_hash: String,
    pub include_actions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub is_incomplete: bool,
    pub actions: Vec<TraceAction>,
    pub transactions_order: Vec<String>,
    pub transactions: HashMap<String, TransactionMessage>,
}

impl Trace {
    fn action_state(&self) -> TransactionState {
        if self.is_incomplete {
            return TransactionState::Pending;
        }
        for action in &self.actions {
            if action.success == Some(false) {
                return TransactionState::Reverted;
            }
        }
        TransactionState::Confirmed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAction {
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub hash: String,
    pub now: i64,
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
