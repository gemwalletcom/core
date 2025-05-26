use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize};
use typeshare::typeshare; // Added from client/model.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct XRPResult<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountResult {
    pub account_data: Option<XRPAccount>,
    pub ledger_current_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccount {
    #[serde(rename = "Balance")]
    pub balance: String,
    #[serde(rename = "Sequence")]
    pub sequence: i32,
    #[serde(rename = "OwnerCount")]
    pub owner_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPFee {
    pub drops: XRPDrops,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPDrops {
    pub minimum_fee: String,
    pub median_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransactionBroadcast {
    pub accepted: Option<bool>,
    pub engine_result_message: Option<String>,
    pub error_exception: Option<String>,
    pub tx_json: Option<XRPTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransaction {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTransactionStatus {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPLatestBlock {
    pub ledger_current_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountObjects<T> {
    pub account_objects: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountAsset {
    pub low_limit: XRPAssetLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAssetLine {
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPTokenId {
    pub issuer: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountLinesResult {
    pub lines: Option<Vec<XRPAccountLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct XRPAccountLine {
    pub account: String,
    pub balance: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerResult<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerCurrent {
    pub ledger_current_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerData {
    pub ledger: Ledger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObjects {
    pub account_objects: Vec<AccountObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObject {
    #[serde(rename = "LowLimit")]
    pub low_limit: AccountObjectLowLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObjectLowLimit {
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub close_time: i64,
    // This refers to the detailed Transaction struct below
    pub transactions: Vec<RpcTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTransaction {
    pub hash: String,
    #[serde(rename = "Fee")]
    pub fee: Option<String>,
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    #[serde(rename = "Sequence")]
    pub sequence: i64,
    pub date: Option<i64>,
    #[serde(rename = "DestinationTag")]
    pub destination_tag: Option<i64>,
    #[serde(rename = "Memos")]
    pub memos: Option<Vec<TransactionMemo>>,
    #[serde(rename = "metaData")]
    pub meta: TransactionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMeta {
    #[serde(rename = "TransactionResult")]
    pub result: String,
    pub delivered_amount: Option<Amount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Str(String),
    Amount(AmountCurrency),
}

impl Amount {
    pub fn as_value_string(&self) -> Option<String> {
        match self {
            Amount::Str(amount) => Some(amount.clone()),
            Amount::Amount(amount) => BigNumberFormatter::value_from_amount(&amount.value, 15),
        }
    }

    pub fn token_id(&self) -> Option<String> {
        match self {
            Amount::Str(_) => None,
            Amount::Amount(amount) => Some(amount.issuer.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountCurrency {
    pub value: String,
    pub issuer: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMemo {
    #[serde(rename = "Memo")]
    pub memo: TransactionMemoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMemoData {
    #[serde(rename = "MemoData")]
    pub data: Option<String>,
}
