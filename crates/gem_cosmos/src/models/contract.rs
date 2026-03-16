use serde::Deserialize;

use super::Coin;

#[derive(Debug, Deserialize)]
pub struct ExecuteContractValue {
    pub sender: String,
    pub contract: String,
    pub msg: String,
    pub funds: Vec<Coin>,
}
