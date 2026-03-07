use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinTransaction {
    pub block_height: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransactionBroacastResult {
    pub error: Option<BitcoinTransactionBroacastError>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BitcoinTransactionBroacastError {
    Plain(String),
    Detailed { message: String },
}

impl BitcoinTransactionBroacastError {
    pub fn message(&self) -> &str {
        match self {
            Self::Plain(s) => s,
            Self::Detailed { message } => message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: i32,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressDetails {
    pub transactions: Option<Vec<Transaction>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub txid: String,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub block_time: i64,
    pub block_height: i64,
    pub vin: Vec<Input>,
    pub vout: Vec<Output>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>, // will be optional for Coinbase Input
    pub value: String,
    pub n: i64,
    pub tx_id: Option<String>, // will be optional for Coinbase Input
    pub vout: Option<i64>,     // will be optional for Coinbase Input
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>,
    pub value: String,
    pub n: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_broadcast_error_string() {
        let json = r#"{"error": "-26: min relay fee not met, 432 < 576"}"#;
        let result: BitcoinTransactionBroacastResult = serde_json::from_str(json).unwrap();

        assert!(result.result.is_none());
        assert_eq!(result.error.unwrap().message(), "-26: min relay fee not met, 432 < 576");
    }

    #[test]
    fn test_deserialize_broadcast_error_object() {
        let json = r#"{"error": {"message": "transaction already in block chain"}}"#;
        let result: BitcoinTransactionBroacastResult = serde_json::from_str(json).unwrap();

        assert!(result.result.is_none());
        assert_eq!(result.error.unwrap().message(), "transaction already in block chain");
    }

    #[test]
    fn test_deserialize_broadcast_success() {
        let json = r#"{"result": "abc123def456"}"#;
        let result: BitcoinTransactionBroacastResult = serde_json::from_str(json).unwrap();

        assert!(result.error.is_none());
        assert_eq!(result.result.unwrap(), "abc123def456");
    }
}
