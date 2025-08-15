use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerResult<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerError {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerCurrent {
    pub ledger_current_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerData {
    pub ledger: Ledger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountObjects {
    pub account_objects: Vec<AccountObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObject {
    #[serde(rename = "LowLimit")]
    pub low_limit: AccountObjectLimit,
    #[serde(rename = "HighLimit")]
    pub high_limit: AccountObjectLimit,
    #[serde(rename = "Balance")]
    pub balance: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub value: String,
}

impl AccountObjectLimit {
    pub fn symbol(&self) -> Option<String> {
        let currency_bytes: Vec<u8> = hex::decode(&self.currency).ok()?;
        String::from_utf8(currency_bytes.into_iter().filter(|b| *b != 0).collect()).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObjectLimit {
    pub currency: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub close_time: i64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub ledger_index: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountLedger {
    pub transactions: Vec<AccountLedgerTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerTransaction {
    pub hash: String,
    pub ledger_index: i64,
    pub tx_json: AccountLedgerTransactionJSON,
    #[serde(rename = "meta")]
    pub meta: TransactionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerTransactionJSON {
    #[serde(rename = "Fee")]
    pub fee: Option<String>,
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "DeliverMax")]
    pub amount: Option<Amount>,
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    pub date: i64,
    #[serde(rename = "DestinationTag")]
    pub destination_tag: Option<i64>,
    #[serde(rename = "Memos")]
    pub memos: Option<Vec<TransactionMemo>>,
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
            Amount::Amount(amount) => BigNumberFormatter::value_from_amount(&amount.value, 15).ok(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_object_symbol_rlusd() {
        let account_object = AccountObjectLimit {
            currency: "524C555344000000000000000000000000000000".to_string(),
            issuer: "".to_string(),
        };
        assert_eq!(account_object.symbol(), Some("RLUSD".to_string()));
    }
}
