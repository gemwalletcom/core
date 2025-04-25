use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TronGridResponse {
    pub result: TronGridResult,
    pub energy_used: u64,
    pub energy_penalty: u64,
    pub constant_result: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TronGridResult {
    Result(TronResult),
    Error(TronErrorResult),
}

#[derive(Deserialize, Debug)]
pub struct TronResult {
    pub result: bool,
}

#[derive(Deserialize, Debug)]
pub struct TronErrorResult {
    pub code: String,
    pub message: String,
}
