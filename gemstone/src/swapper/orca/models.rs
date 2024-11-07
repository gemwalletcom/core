use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAccount {
    pub account: ProgramAccountData,
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAccountData {
    pub data: Vec<String>,
    pub owner: String,
    pub space: u64,
}
