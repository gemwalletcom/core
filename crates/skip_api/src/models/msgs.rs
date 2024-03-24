use crate::models::fee::EstimatedFee;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgRequest {
    pub source_asset_denom: String,
    pub source_asset_chain_id: String,
    pub dest_asset_denom: String,
    pub dest_asset_chain_id: String,
    pub amount_in: String,
    pub amount_out: String,
    pub operations: serde_json::Value,
    pub address_list: Vec<String>,
    pub slippage_tolerance_percent: String,
    pub client_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgResponse {
    pub msgs: Vec<Msg>,
    pub txs: Vec<Tx>,
    pub estimated_fees: Vec<EstimatedFee>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Msg {
    pub multi_chain_msg: MultiChainMsg,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiChainMsg {
    pub chain_id: String,
    pub path: Vec<String>,
    pub msg: String,
    pub msg_type_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tx {
    pub cosmos_tx: CosmosTx,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosTx {
    pub chain_id: String,
    pub path: Vec<String>,
    pub msgs: Vec<CosmosTxMsg>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmosTxMsg {
    pub msg: String,
    pub msg_type_url: String,
}
