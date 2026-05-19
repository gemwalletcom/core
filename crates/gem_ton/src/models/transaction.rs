use std::collections::HashMap;

use num_bigint::BigUint;
use primitives::TransactionState;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

use crate::address::Address;

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
        self.traces.first()?.root_transaction()
    }

    pub fn action_state(&self) -> Option<TransactionState> {
        self.traces.first().map(Trace::action_state)
    }

    pub fn has_actions(&self) -> bool {
        self.traces.first().is_some_and(Trace::has_actions)
    }
}

#[derive(Debug, Serialize)]
pub struct TraceByMessageQuery {
    pub msg_hash: String,
    pub include_actions: bool,
}

#[derive(Debug, Serialize)]
pub struct TraceByTransactionQuery {
    pub tx_hash: String,
    pub include_actions: bool,
}

#[derive(Debug, Serialize)]
pub struct TraceByBlockQuery {
    pub mc_seqno: u64,
    pub include_actions: bool,
    pub limit: usize,
    pub offset: usize,
    pub sort: &'static str,
}

#[derive(Debug, Serialize)]
pub struct TraceByAddressQuery {
    pub account: String,
    pub include_actions: bool,
    pub limit: usize,
    pub offset: usize,
    pub sort: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub is_incomplete: bool,
    pub actions: Vec<TraceAction>,
    pub transactions_order: Vec<String>,
    pub transactions: HashMap<String, TransactionMessage>,
}

impl Trace {
    pub fn root_transaction(&self) -> Option<&TransactionMessage> {
        let transaction_id = self.transactions_order.first()?;
        self.transactions.get(transaction_id)
    }

    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn action_state(&self) -> TransactionState {
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

pub const TRACE_ACTION_JETTON_SWAP: &str = "jetton_swap";
pub const TRACE_ACTION_JETTON_TRANSFER: &str = "jetton_transfer";
pub const TRACE_ACTION_NFT_TRANSFER: &str = "nft_transfer";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAction {
    pub success: Option<bool>,
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonSwapDetails {
    pub dex: Option<String>,
    pub sender: String,
    pub asset_in: Option<String>,
    pub asset_out: Option<String>,
    pub dex_incoming_transfer: SwapTransfer,
    pub dex_outgoing_transfer: SwapTransfer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwapTransfer {
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonTransferDetails {
    pub asset: String,
    pub sender: String,
    pub receiver: String,
    pub amount: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NftTransferDetails {
    pub nft_collection: Address,
    pub nft_item: Address,
    pub old_owner: Address,
    pub new_owner: Address,
    pub comment: Option<String>,
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
    #[serde(alias = "opcode")]
    pub op_code: Option<String>,
    pub comment: Option<String>,
    pub decoded_body: Option<DecodedBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInMessage {
    pub source: Option<String>,
    pub destination: String,
    pub value: Option<String>,
    pub opcode: Option<String>,
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
