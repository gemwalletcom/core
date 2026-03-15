use serde::Deserialize;

use super::Coin;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteContractValue {
    #[serde(default)]
    pub sender: String,
    pub contract: String,
    pub msg: String,
    pub funds: Vec<Coin>,
}
