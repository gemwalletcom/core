use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Msg {
    pub multi_chain_msg: MultiChainMsg,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MultiChainMsg {
    pub chain_id: String,
    pub path: Vec<String>,
    pub msg: String,
    pub msg_type_url: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
