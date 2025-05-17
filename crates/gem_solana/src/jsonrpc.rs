use std::fmt::Display;

use crate::model::Filter;

pub const ENCODING_BASE64: &str = "base64";
pub const ENCODING_BASE58: &str = "base58";

pub enum SolanaRpc {
    GetProgramAccounts(String, Vec<Filter>),
    GetAccountInfo(String),
    GetMultipleAccounts(Vec<String>),
    GetEpochInfo,
}

impl Display for SolanaRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolanaRpc::GetProgramAccounts(_, _) => write!(f, "getProgramAccounts"),
            SolanaRpc::GetAccountInfo(_) => write!(f, "getAccountInfo"),
            SolanaRpc::GetMultipleAccounts(_) => write!(f, "getMultipleAccounts"),
            SolanaRpc::GetEpochInfo => write!(f, "getEpochInfo"),
        }
    }
}
