use std::fmt::Display;

use crate::model::{Configuration, Filter};
use gem_jsonrpc::types::{JsonRpcRequest, JsonRpcRequestConvert};
use serde_json::Value;

pub const ENCODING_BASE64: &str = "base64";
pub const ENCODING_BASE58: &str = "base58";

pub enum SolanaRpc {
    GetProgramAccounts(String, Vec<Filter>),
    GetAccountInfo(String),
    GetMultipleAccounts(Vec<String>),
    GetEpochInfo,
    GetLatestBlockhash,
}

impl Display for SolanaRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolanaRpc::GetProgramAccounts(_, _) => write!(f, "getProgramAccounts"),
            SolanaRpc::GetAccountInfo(_) => write!(f, "getAccountInfo"),
            SolanaRpc::GetMultipleAccounts(_) => write!(f, "getMultipleAccounts"),
            SolanaRpc::GetEpochInfo => write!(f, "getEpochInfo"),
            SolanaRpc::GetLatestBlockhash => write!(f, "getLatestBlockhash"),
        }
    }
}

impl JsonRpcRequestConvert for SolanaRpc {
    fn to_req(&self, id: u64) -> JsonRpcRequest {
        let val = self;
        let method = val.to_string();
        let default_config = Configuration::default();

        let params: Vec<Value> = match val {
            SolanaRpc::GetProgramAccounts(program, filters) => vec![
                Value::String(program.into()),
                serde_json::to_value(Configuration::new(filters.to_vec())).unwrap(),
            ],
            SolanaRpc::GetAccountInfo(program) => vec![Value::String(program.into()), serde_json::to_value(default_config).unwrap()],
            SolanaRpc::GetMultipleAccounts(accounts) => vec![
                Value::Array(accounts.iter().map(|x| serde_json::to_value(x).unwrap()).collect()),
                serde_json::to_value(default_config).unwrap(),
            ],
            SolanaRpc::GetEpochInfo | SolanaRpc::GetLatestBlockhash => vec![],
        };

        JsonRpcRequest::new(id, &method, params)
    }
}
