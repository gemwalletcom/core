use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub data: Vec<String>,
    pub owner: String,
    pub space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueResult<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueData<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedData<T> {
    pub parsed: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInfo<T> {
    pub info: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTokenInfo {
    pub decimals: i32,
    pub is_initialized: bool,
    pub mint_authority: String,
    pub supply: String,
}

pub type SolanaParsedTokenInfo = ValueResult<ValueData<ParsedData<ParsedInfo<ParsedTokenInfo>>>>;
