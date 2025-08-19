use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chainhead {
    pub last: BlockInfo,
    #[serde(rename = "init")]
    pub initial: BlockInfo,
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
pub struct AccountInfo {
    pub balance: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonResult<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub seqno: i64,
    pub root_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonWalletInfo {
    pub wallet: bool,
    pub balance: String,
    pub account_state: String,
    pub wallet_type: String,
    pub seqno: Option<i64>,
    pub last_transaction_id: Option<serde_json::Value>,
    pub wallet_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonJettonBalance {
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGetMethod {
    pub stack: Vec<Vec<StackItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StackItem {
    String(String),
    Cell(Cell),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub bytes: String,
    pub object: CellObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellObject {
    pub data: CellData,
    pub refs: Vec<String>,
    pub special: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub b64: String,
    pub len: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonBroadcastTransaction {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWalletsResponse {
    pub jetton_wallets: Vec<JettonWallet>,
    pub address_book: serde_json::Value,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWallet {
    pub address: String,
    pub balance: String,
    pub owner: String,
    pub jetton: String,
    pub last_transaction_lt: String,
    pub code_hash: String,
    pub data_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonMessageTransactions {
    pub transactions: Vec<TonTransactionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonTransactionMessage {
    pub hash: String,
    pub total_fees: String,
    pub description: Option<TransactionDescription>,
    pub out_msgs: Vec<TonOutMessage>,
    #[serde(default)]
    pub account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDescription {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub aborted: bool,
    pub destroyed: bool,
    pub credit_first: bool,
    pub storage_ph: Option<StoragePhase>,
    pub compute_ph: Option<ComputePhase>,
    pub action: Option<ActionPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePhase {
    pub storage_fees_collected: String,
    pub status_change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePhase {
    pub skipped: bool,
    pub success: bool,
    pub msg_state_used: bool,
    pub account_activated: bool,
    pub gas_fees: String,
    pub gas_used: String,
    pub gas_limit: String,
    pub gas_credit: String,
    pub mode: i32,
    pub exit_code: i32,
    pub vm_steps: i64,
    pub vm_init_state_hash: String,
    pub vm_final_state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPhase {
    pub success: bool,
    pub valid: bool,
    pub no_funds: bool,
    pub status_change: String,
    pub total_fwd_fees: String,
    pub total_action_fees: String,
    pub result_code: i32,
    pub tot_actions: i32,
    pub spec_actions: i32,
    pub skipped_actions: i32,
    pub msgs_created: i32,
    pub action_list_hash: String,
    pub tot_msg_size: MessageSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSize {
    pub cells: String,
    pub bits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRef {
    pub workchain: i32,
    pub shard: String,
    pub seqno: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub hash: String,
    pub body: String,
    pub decoded: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonTransactionInMessage {
    pub hash: String,
    pub source: Option<String>,
    pub destination: String,
    pub value: Option<String>,
    pub value_extra_currencies: Option<serde_json::Value>,
    pub fwd_fee: Option<String>,
    pub ihr_fee: Option<String>,
    pub created_lt: Option<String>,
    pub created_at: Option<String>,
    pub opcode: Option<String>,
    pub ihr_disabled: Option<bool>,
    pub bounce: Option<bool>,
    pub bounced: Option<bool>,
    pub import_fee: String,
    pub message_content: MessageContent,
    pub init_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonOutMessage {
    pub hash: String,
    pub source: String,
    pub destination: String,
    pub value: String,
    pub value_extra_currencies: serde_json::Value,
    pub fwd_fee: String,
    pub ihr_fee: String,
    pub created_lt: String,
    pub created_at: String,
    pub opcode: Option<String>,
    pub ihr_disabled: bool,
    pub bounce: bool,
    pub bounced: bool,
    pub import_fee: Option<String>,
    pub message_content: MessageContent,
    pub init_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub hash: String,
    pub balance: Option<String>,
    pub extra_currencies: Option<serde_json::Value>,
    pub account_status: Option<String>,
    pub frozen_hash: Option<String>,
    pub data_hash: Option<String>,
    pub code_hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_wallet_info_deserialization() {
        let response: TonResult<TonWalletInfo> = serde_json::from_str(include_str!("../../testdata/wallet_information.json")).unwrap();
        
        assert!(response.ok);
        let wallet_info = response.result;
        assert!(wallet_info.wallet);
        assert_eq!(wallet_info.balance, "62709394797");
        assert_eq!(wallet_info.account_state, "active");
        assert_eq!(wallet_info.wallet_type, "wallet v4 r2");
        assert_eq!(wallet_info.seqno, Some(140));
        assert_eq!(wallet_info.wallet_id, Some(698983191));
        assert!(wallet_info.last_transaction_id.is_some());
    }
}
