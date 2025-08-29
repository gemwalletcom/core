use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponseLegacy {
    pub block: BlockLegacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockLegacy {
    pub header: Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub chain_id: String,
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoResponse {
    pub default_node_info: NodeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syncing {
    pub syncing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub data: BlockData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub txs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationResponse {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub inflation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualProvisionsResponse {
    pub annual_provisions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyResponse {
    pub amount: SupplyAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyAmount {
    pub denom: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
}
