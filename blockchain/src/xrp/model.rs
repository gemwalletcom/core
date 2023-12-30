use serde::{Deserialize, Serialize};

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
pub struct Ledger {
    pub close_time: i64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
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
    pub fn as_value_string(&self) -> String {
        match self {
            Amount::Str(amount) => amount.to_string(),
            Amount::Amount(amount) => amount.value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountCurrency {
    pub value: String,
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
