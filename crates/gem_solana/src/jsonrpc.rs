use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub commitment: &'static str,
    pub encoding: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter>,
}

impl Configuration {
    pub fn new(filters: Vec<Filter>) -> Self {
        Self {
            commitment: "confirmed",
            encoding: ENCODING_BASE64,
            filters,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            commitment: "confirmed",
            encoding: ENCODING_BASE64,
            filters: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub memcmp: Memcmp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memcmp {
    pub offset: u8,
    pub bytes: String,
    pub encoding: String,
}
