use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoBlock {
    pub number: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoBlockData {
    pub cardano: CardanoBlockTip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoBlockTip {
    pub tip: CardanoBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoGenesisData {
    pub genesis: CardanoGenesis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoGenesis {
    pub shelley: CardanoGenesisShelley,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardanoGenesisShelley {
    pub network_magic: i32,
}
