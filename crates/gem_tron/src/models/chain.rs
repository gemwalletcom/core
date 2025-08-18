use serde::{Deserialize, Serialize};

type Int64 = i64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronChainParameters {
    #[serde(rename = "chainParameter")]
    pub chain_parameter: Vec<TronChainParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronChainParameter {
    pub key: String,
    pub value: Option<Int64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TronChainParameterKey {
    #[serde(rename = "getCreateNewAccountFeeInSystemContract")]
    GetCreateNewAccountFeeInSystemContract,
    #[serde(rename = "getCreateAccountFee")]
    GetCreateAccountFee,
    #[serde(rename = "getEnergyFee")]
    GetEnergyFee,
}
