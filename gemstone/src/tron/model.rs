use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TronNodeResponse {
    pub result: TronNodeResult,
    pub energy_used: u64,
    pub energy_penalty: u64,
    pub constant_result: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TronNodeResult {
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
