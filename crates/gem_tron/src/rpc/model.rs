use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_header: BlockHeader,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockTransactions {
    pub block_header: BlockHeader,
    pub transactions: Option<Vec<Transaction>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeader {
    pub raw_data: BlockHeaderData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeaderData {
    pub number: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub ret: Vec<ContractRet>,
    pub raw_data: TransactionData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractRet {
    #[serde(rename = "contractRet")]
    pub contract_ret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionData {
    pub contract: Vec<Contract>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Contract {
    #[serde(rename = "type")]
    pub contract_type: String,
    pub parameter: ContractParameter,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameter {
    pub type_url: String,
    pub value: ContractParameterValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameterValue {
    pub amount: Option<i64>,
    pub owner_address: Option<String>,
    pub to_address: Option<String>,
    pub contract_address: Option<String>,
}

pub type BlockTransactionsInfo = Vec<TransactionReceiptData>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceiptData {
    pub id: String,
    pub fee: Option<i64>,
    #[serde(rename = "blockNumber")]
    pub block_number: i64,
    #[serde(rename = "blockTimeStamp")]
    pub block_time_stamp: i64,
    pub receipt: TransactionReceipt,
    pub log: Option<Vec<TronLog>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceipt {
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TronLog {
    pub topics: Option<Vec<String>>,
    pub data: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TriggerConstantContractRequest {
    pub owner_address: String,
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: String,
    pub visible: bool,
}

#[derive(Deserialize, Debug)]
pub struct TriggerConstantContractResponse {
    #[serde(default)]
    pub constant_result: Vec<String>,
    pub result: Option<TriggerContractResult>,
}

#[derive(Deserialize, Debug)]
pub struct TriggerContractResult {
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessesList {
    pub witnesses: Vec<WitnessAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WitnessAccount {
    pub address: String,
    pub vote_count: Option<i64>,
    pub url: String,
    pub is_jobs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainParametersResponse {
    pub chain_parameter: Vec<ChainParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParameter {
    pub key: String,
    pub value: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionBroadcast {
    #[serde(rename = "txid")]
    pub txid: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}
