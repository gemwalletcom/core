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
