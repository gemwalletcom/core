use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpDex {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub deployer: Option<String>,
    pub oracle_updater: Option<String>,
    pub chain_id: Option<u64>,
    pub is_active: Option<bool>,
}
