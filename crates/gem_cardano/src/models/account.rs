use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoBalance {
    pub address: String,
    pub tx_hash: String,
    pub index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoBalanceResponse {
    pub utxos: CardanoAggregateBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoAggregateBalance {
    pub aggregate: CardanoAggregateSum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoAggregateSum {
    pub sum: CardanoAggregateSumValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoAggregateSumValue {
    pub value: Option<String>,
}

