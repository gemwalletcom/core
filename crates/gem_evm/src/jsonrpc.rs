use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionObject {
    pub from: String,
    pub to: String,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub value: Option<String>,
    pub data: Option<Vec<u8>>,
}

impl TransactionObject {
    pub fn new_call(from: &str, to: &str, data: Option<Vec<u8>>) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            gas: None,
            gas_price: None,
            value: None,
            data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockParameter {
    // hexadecimal block number
    Number(&'static str),
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
}

impl From<&BlockParameter> for &'static str {
    fn from(val: &BlockParameter) -> Self {
        match val {
            BlockParameter::Number(val) => val,
            BlockParameter::Latest => "latest",
            BlockParameter::Earliest => "earliest",
            BlockParameter::Pending => "pending",
            BlockParameter::Safe => "safe",
            BlockParameter::Finalized => "finalized",
        }
    }
}

impl From<&BlockParameter> for serde_json::Value {
    fn from(val: &BlockParameter) -> Self {
        let str: &str = val.into();
        serde_json::Value::String(str.to_string())
    }
}

#[derive(Debug)]
pub enum EthereumRpc {
    GasPrice,
    GetBalance(&'static str),
    Call(TransactionObject, BlockParameter),
}

impl EthereumRpc {
    pub fn method_name(&self) -> &'static str {
        match self {
            EthereumRpc::GasPrice => "eth_gasPrice",
            EthereumRpc::GetBalance(_) => "eth_getBalance",
            EthereumRpc::Call(_, _) => "eth_call",
        }
    }
}
