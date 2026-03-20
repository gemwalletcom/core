use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetOrder {
    PriceChange24hAsc,
    PriceChange24hDesc,
}
