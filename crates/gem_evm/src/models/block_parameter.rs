use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EthereumBlockParameter {
    Latest,
    Earliest,
    Pending,
    Finalized,
    Safe,
}
